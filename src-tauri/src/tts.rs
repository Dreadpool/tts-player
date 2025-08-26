use reqwest;
use serde_json::{json, Value};
use std::time::Duration;
use tokio::time::sleep;
use chrono::{DateTime, Utc};
use crate::database::{Database, UsageRecord, UserInfo};

#[derive(Debug)]
pub enum TTSError {
    Authentication(String),
    RateLimit(Option<u64>),
    ValidationError(String),
    NetworkError(String),
    UnknownError(String),
}

impl std::fmt::Display for TTSError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TTSError::Authentication(msg) => write!(f, "Authentication error: {}", msg),
            TTSError::RateLimit(retry_after) => {
                if let Some(seconds) = retry_after {
                    write!(f, "Rate limit exceeded. Retry after {} seconds", seconds)
                } else {
                    write!(f, "Rate limit exceeded")
                }
            }
            TTSError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            TTSError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            TTSError::UnknownError(msg) => write!(f, "Unknown error: {}", msg),
        }
    }
}

impl std::error::Error for TTSError {}

impl From<TTSError> for String {
    fn from(error: TTSError) -> String {
        error.to_string()
    }
}

pub struct TTSService {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    database: Option<Database>,
}

impl TTSService {
    pub fn new(api_key: &str, base_url: &str) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();
            
        Self {
            client,
            api_key: api_key.to_string(),
            base_url: base_url.to_string(),
            database: None,
        }
    }

    pub async fn with_database(api_key: &str, base_url: &str) -> Result<Self, TTSError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();

        let database = Database::new().await
            .map_err(|e| TTSError::UnknownError(format!("Database error: {}", e)))?;
            
        Ok(Self {
            client,
            api_key: api_key.to_string(),
            base_url: base_url.to_string(),
            database: Some(database),
        })
    }

    pub async fn validate_text(&self, text: &str) -> Result<(), TTSError> {
        if text.trim().is_empty() {
            return Err(TTSError::ValidationError("Text cannot be empty".to_string()));
        }
        
        if text.len() > 5000 {
            return Err(TTSError::ValidationError("Text too long (max 5000 characters)".to_string()));
        }
        
        Ok(())
    }

    pub fn is_valid_voice(&self, voice_id: &str) -> bool {
        // Accept any non-empty voice ID since ElevenLabs has many voices with complex IDs
        !voice_id.trim().is_empty()
    }

    pub async fn generate_speech(&self, text: &str, voice_id: &str) -> Result<Vec<u8>, TTSError> {
        let url = format!("{}/v1/text-to-speech/{}", self.base_url, voice_id);
        
        let request_body = json!({
            "text": text,
            "model_id": "eleven_multilingual_v2"
        });

        let response = self.client
            .post(&url)
            .header("xi-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| TTSError::NetworkError(e.to_string()))?;

        match response.status() {
            reqwest::StatusCode::OK => {
                let audio_data = response.bytes().await
                    .map_err(|e| TTSError::NetworkError(e.to_string()))?;
                Ok(audio_data.to_vec())
            }
            reqwest::StatusCode::UNAUTHORIZED => {
                let error_text = response.text().await.unwrap_or_default();
                Err(TTSError::Authentication(error_text))
            }
            reqwest::StatusCode::TOO_MANY_REQUESTS => {
                let retry_after = response.headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse().ok());
                Err(TTSError::RateLimit(retry_after))
            }
            status => {
                let error_text = response.text().await.unwrap_or_default();
                Err(TTSError::UnknownError(format!("HTTP {}: {}", status, error_text)))
            }
        }
    }

    pub async fn generate_speech_with_retry(&self, text: &str, voice_id: &str) -> Result<Vec<u8>, TTSError> {
        const MAX_RETRIES: u32 = 3;
        const BASE_DELAY_MS: u64 = 1000;
        
        for attempt in 0..MAX_RETRIES {
            match self.generate_speech(text, voice_id).await {
                Ok(audio_data) => return Ok(audio_data),
                Err(TTSError::RateLimit(_)) => return Err(TTSError::RateLimit(None)), // Don't retry rate limits
                Err(TTSError::Authentication(_)) => return Err(TTSError::Authentication("API key invalid".to_string())), // Don't retry auth errors
                Err(err) if attempt == MAX_RETRIES - 1 => return Err(err), // Last attempt
                Err(_) => {
                    // Exponential backoff
                    let delay = Duration::from_millis(BASE_DELAY_MS * 2_u64.pow(attempt));
                    sleep(delay).await;
                }
            }
        }
        
        unreachable!()
    }

    pub async fn get_user_info(&self) -> Result<UserInfo, TTSError> {
        // Check cached info first (if less than 5 minutes old)
        if let Some(db) = &self.database {
            if let Ok(Some(cached_info)) = db.get_cached_user_info().await {
                let age = Utc::now().signed_duration_since(cached_info.last_updated);
                if age.num_minutes() < 5 {
                    return Ok(cached_info);
                }
            }
        }

        // Fetch fresh user info from API
        let url = format!("{}/v1/user", self.base_url);
        
        let response = self.client
            .get(&url)
            .header("xi-api-key", &self.api_key)
            .send()
            .await
            .map_err(|e| TTSError::NetworkError(e.to_string()))?;

        match response.status() {
            reqwest::StatusCode::OK => {
                let user_data: Value = response.json().await
                    .map_err(|e| TTSError::NetworkError(e.to_string()))?;
                
                let user_info = UserInfo {
                    subscription_tier: user_data["subscription"]["tier"].as_str().unwrap_or("unknown").to_string(),
                    character_limit: user_data["subscription"]["character_limit"].as_i64().unwrap_or(0) as i32,
                    character_used: user_data["subscription"]["character_count"].as_i64().unwrap_or(0) as i32,
                    characters_remaining: user_data["subscription"]["character_limit"].as_i64().unwrap_or(0) as i32 
                        - user_data["subscription"]["character_count"].as_i64().unwrap_or(0) as i32,
                    reset_date: Utc::now(), // This would need to be parsed from the API response
                    last_updated: Utc::now(),
                };

                // Cache the user info
                if let Some(db) = &self.database {
                    let _ = db.cache_user_info(&user_info).await;
                }

                Ok(user_info)
            }
            reqwest::StatusCode::UNAUTHORIZED => {
                Err(TTSError::Authentication("Invalid API key".to_string()))
            }
            status => {
                let error_text = response.text().await.unwrap_or_default();
                Err(TTSError::UnknownError(format!("HTTP {}: {}", status, error_text)))
            }
        }
    }

    pub async fn track_usage(&self, text: &str, voice_id: &str, model_id: &str, success: bool, error_message: Option<String>) -> Result<(), TTSError> {
        if let Some(db) = &self.database {
            let record = UsageRecord {
                id: None,
                timestamp: Utc::now(),
                text: if text.len() > 100 { 
                    // Store only first 100 chars to save space
                    format!("{}...", &text[..97])
                } else { 
                    text.to_string() 
                },
                character_count: text.len() as i32,
                voice_id: voice_id.to_string(),
                model_id: model_id.to_string(),
                success,
                error_message,
            };

            db.record_usage(&record).await
                .map_err(|e| TTSError::UnknownError(format!("Database error: {}", e)))?;
        }
        Ok(())
    }

    pub async fn generate_speech_tracked(&self, text: &str, voice_id: &str) -> Result<Vec<u8>, TTSError> {
        let model_id = "eleven_multilingual_v2";
        let start_time = Utc::now();
        
        match self.generate_speech(text, voice_id).await {
            Ok(audio_data) => {
                // Track successful usage
                self.track_usage(text, voice_id, model_id, true, None).await?;
                Ok(audio_data)
            }
            Err(error) => {
                // Track failed usage
                let error_msg = error.to_string();
                self.track_usage(text, voice_id, model_id, false, Some(error_msg.clone())).await?;
                Err(error)
            }
        }
    }

    pub async fn get_usage_stats(&self, days: i32) -> Result<crate::database::UsageStats, TTSError> {
        if let Some(db) = &self.database {
            db.get_usage_stats(days).await
                .map_err(|e| TTSError::UnknownError(format!("Database error: {}", e)))
        } else {
            Err(TTSError::UnknownError("Database not available".to_string()))
        }
    }

    pub async fn get_usage_history(&self, limit: i32, days: Option<i32>) -> Result<Vec<UsageRecord>, TTSError> {
        if let Some(db) = &self.database {
            db.get_usage_records(limit, days).await
                .map_err(|e| TTSError::UnknownError(format!("Database error: {}", e)))
        } else {
            Err(TTSError::UnknownError("Database not available".to_string()))
        }
    }

    pub fn count_characters(&self, text: &str) -> i32 {
        text.len() as i32
    }

    pub fn estimate_usage_cost(&self, character_count: i32, subscription_tier: &str) -> f64 {
        // Rough cost estimation based on ElevenLabs pricing
        match subscription_tier {
            "starter" => 0.0, // Free tier
            "creator" => character_count as f64 * 0.00003, // $0.30 per 10K chars
            "pro" => character_count as f64 * 0.00003,
            "scale" => character_count as f64 * 0.00002, // Volume discount
            _ => 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{Matcher, Server};
    use tokio_test;

    #[tokio::test]
    async fn test_text_validation() {
        let service = TTSService::new("test-key", "https://api.elevenlabs.io");
        
        // Empty text should fail
        assert!(service.validate_text("").await.is_err());
        assert!(service.validate_text("   ").await.is_err());
        
        // Valid text should pass
        assert!(service.validate_text("Hello world").await.is_ok());
        
        // Too long text should fail
        let long_text = "a".repeat(5001);
        assert!(service.validate_text(&long_text).await.is_err());
    }

    #[test]
    fn test_voice_validation() {
        let service = TTSService::new("test-key", "https://api.elevenlabs.io");
        
        assert!(service.is_valid_voice("rachel"));
        assert!(service.is_valid_voice("adam"));
        assert!(service.is_valid_voice("bella"));
        
        assert!(!service.is_valid_voice("invalid"));
        assert!(!service.is_valid_voice(""));
    }
}