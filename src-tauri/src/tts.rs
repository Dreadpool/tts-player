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
        
        if text.len() > 4096 {
            return Err(TTSError::ValidationError("Text too long (max 4096 characters)".to_string()));
        }
        
        Ok(())
    }

    pub fn is_valid_voice(&self, voice_id: &str) -> bool {
        // List of OpenAI TTS voice IDs
        const VALID_VOICE_IDS: &[&str] = &[
            "alloy",   // Neutral, versatile
            "echo",    // Male voice
            "fable",   // British accent
            "onyx",    // Deep male voice
            "nova",    // Natural female voice
            "shimmer", // Expressive female
        ];
        
        let voice_id = voice_id.trim();
        !voice_id.is_empty() && VALID_VOICE_IDS.contains(&voice_id)
    }

    pub async fn generate_speech(&self, text: &str, voice_id: &str) -> Result<Vec<u8>, TTSError> {
        let url = format!("{}/v1/audio/speech", self.base_url);
        
        let request_body = json!({
            "model": "tts-1-hd",
            "input": text,
            "voice": voice_id,
            "response_format": "mp3"
        });

        let response = self.client
            .post(&url)
            .header("Authorization", &format!("Bearer {}", self.api_key))
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

    pub async fn generate_speech_with_model(&self, text: &str, voice_id: &str, model: &str) -> Result<Vec<u8>, TTSError> {
        let url = format!("{}/v1/audio/speech", self.base_url);
        
        let request_body = json!({
            "model": model,
            "input": text,
            "voice": voice_id,
            "response_format": "mp3"
        });

        let response = self.client
            .post(&url)
            .header("Authorization", &format!("Bearer {}", self.api_key))
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
        // OpenAI TTS is pay-per-use, no subscription tiers or limits
        // Get local usage data from database instead
        let character_used = if let Some(db) = &self.database {
            match db.get_usage_stats(30).await { // Get last 30 days
                Ok(stats) => stats.total_characters,
                Err(_) => 0,
            }
        } else {
            0
        };

        let user_info = UserInfo {
            subscription_tier: "Pay-per-use".to_string(),
            character_limit: -1, // Unlimited
            character_used: character_used as i32,
            characters_remaining: -1, // Unlimited
            reset_date: Utc::now(), // Not applicable for pay-per-use
            last_updated: Utc::now(),
        };

        // Cache the user info
        if let Some(db) = &self.database {
            let _ = db.cache_user_info(&user_info).await;
        }

        Ok(user_info)
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
        let model_id = "tts-1-hd"; // OpenAI high-quality model
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

    pub fn estimate_usage_cost(&self, character_count: i32, model: &str) -> f64 {
        // OpenAI TTS pricing (pay-per-use)
        match model {
            "tts-1" => character_count as f64 * 0.000015,    // $15 per 1M characters
            "tts-1-hd" => character_count as f64 * 0.00003,  // $30 per 1M characters
            _ => character_count as f64 * 0.00003, // Default to HD pricing
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