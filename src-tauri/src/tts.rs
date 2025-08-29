use reqwest;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;
use chrono::Utc;
use crate::database::{Database, UsageRecord, UserInfo};
use std::process::Command;
use std::io::{Write, Read};

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
            .timeout(Duration::from_secs(120))
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
            .timeout(Duration::from_secs(120))
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
        
        // No max length check - we'll handle long text by chunking
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
        // For long text, use chunking with proper concatenation
        if text.len() > 4000 {
            eprintln!("[TTS] Text is {} characters, using chunked generation", text.len());
            // Check if FFmpeg is available
            match Command::new("which").arg("ffmpeg").output() {
                Ok(output) if output.status.success() => {
                    eprintln!("[TTS] FFmpeg found, using concatenation");
                    return self.generate_speech_with_ffmpeg_concat(text, voice_id).await;
                }
                _ => {
                    eprintln!("[TTS] FFmpeg not found, falling back to simple truncation");
                    // Fallback: just use the first 4000 characters
                    let truncated = if text.len() > 4000 {
                        &text[..4000]
                    } else {
                        text
                    };
                    eprintln!("[TTS] WARNING: Text truncated to {} characters", truncated.len());
                }
            }
        }
        
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

    // Generate speech for long text using proper FFmpeg concatenation
    async fn generate_speech_with_ffmpeg_concat(&self, text: &str, voice_id: &str) -> Result<Vec<u8>, TTSError> {
        const MAX_CHUNK_SIZE: usize = 3800; // Safe margin under 4096
        
        let chunks = self.split_text_semantically(text, MAX_CHUNK_SIZE);
        eprintln!("Split text into {} chunks", chunks.len());
        
        if chunks.is_empty() {
            return Err(TTSError::ValidationError("No valid text chunks found".to_string()));
        }
        
        // Generate audio for each chunk and save to temp files
        let mut temp_files = Vec::new();
        
        for (i, chunk) in chunks.iter().enumerate() {
            eprintln!("[TTS] Generating audio for chunk {} of {} ({} chars)", i + 1, chunks.len(), chunk.len());
            eprintln!("[TTS] Chunk {} preview: {}...", i + 1, &chunk.chars().take(50).collect::<String>());
            
            // Add delay between API calls to avoid rate limiting
            if i > 0 {
                sleep(Duration::from_millis(200)).await;
            }
            
            // Generate audio for this chunk
            let url = format!("{}/v1/audio/speech", self.base_url);
            let request_body = json!({
                "model": "tts-1-hd",
                "input": chunk,
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
                .map_err(|e| {
                    eprintln!("[TTS] Failed to send request for chunk {}: {}", i + 1, e);
                    TTSError::NetworkError(format!("Failed to send request: {}", e))
                })?;

            let status = response.status();
            eprintln!("[TTS] Chunk {} response status: {}", i + 1, status);
            
            // Read the response body as bytes first
            let body_bytes = response.bytes().await
                .map_err(|e| {
                    eprintln!("[TTS] Failed to read response body for chunk {}: {}", i + 1, e);
                    TTSError::NetworkError(format!("Failed to read response: {}", e))
                })?;
            
            // Check if we got an error response
            if !status.is_success() {
                let error_text = String::from_utf8_lossy(&body_bytes);
                eprintln!("[TTS] API error for chunk {}: HTTP {} - {}", i + 1, status, error_text);
                return Err(TTSError::UnknownError(format!("HTTP {}: {}", status, error_text)));
            }
            
            let audio_data = body_bytes;
            
            eprintln!("[TTS] Chunk {} generated {} bytes", i + 1, audio_data.len());
            
            // Write to temp file with .mp3 extension
            let mut temp_file = tempfile::Builder::new()
                .suffix(".mp3")
                .tempfile()
                .map_err(|e| TTSError::NetworkError(format!("Failed to create temp file: {}", e)))?;
            temp_file.write_all(&audio_data)
                .map_err(|e| TTSError::NetworkError(format!("Failed to write temp file: {}", e)))?;
            temp_file.flush()
                .map_err(|e| TTSError::NetworkError(format!("Failed to flush temp file: {}", e)))?;
            
            temp_files.push(temp_file);
        }
        
        // If only one chunk, return it directly
        if temp_files.len() == 1 {
            let mut buffer = Vec::new();
            std::fs::File::open(temp_files[0].path())
                .and_then(|mut f| std::io::Read::read_to_end(&mut f, &mut buffer))
                .map_err(|e| TTSError::NetworkError(format!("Failed to read temp file: {}", e)))?;
            return Ok(buffer);
        }
        
        // Concatenate using ffmpeg
        eprintln!("[TTS] Concatenating {} audio files with ffmpeg", temp_files.len());
        
        // Create a list file for ffmpeg concat with .txt extension
        let mut list_file = tempfile::Builder::new()
            .suffix(".txt")
            .tempfile()
            .map_err(|e| {
                eprintln!("[TTS] Failed to create list file: {}", e);
                TTSError::NetworkError(format!("Failed to create list file: {}", e))
            })?;
        
        for temp_file in &temp_files {
            writeln!(list_file, "file '{}'" , temp_file.path().display())
                .map_err(|e| TTSError::NetworkError(format!("Failed to write list file: {}", e)))?;
        }
        list_file.flush()
            .map_err(|e| TTSError::NetworkError(format!("Failed to flush list file: {}", e)))?;
        
        // Create output temp file with .mp3 extension
        let output_file = tempfile::Builder::new()
            .suffix(".mp3")
            .tempfile()
            .map_err(|e| TTSError::NetworkError(format!("Failed to create output file: {}", e)))?;
        
        // Log the list file for debugging
        eprintln!("[TTS] List file path: {}", list_file.path().display());
        eprintln!("[TTS] Output file path: {}", output_file.path().display());
        
        // Run ffmpeg to concatenate
        eprintln!("[TTS] Running ffmpeg concat command");
        let output = Command::new("ffmpeg")
            .args(&[
                "-f", "concat",
                "-safe", "0",
                "-i", list_file.path().to_str().unwrap(),
                "-c", "copy",
                "-y",
                output_file.path().to_str().unwrap()
            ])
            .output()
            .map_err(|e| {
                eprintln!("[TTS] Failed to run ffmpeg: {}", e);
                TTSError::NetworkError(format!("Failed to run ffmpeg: {}", e))
            })?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            eprintln!("[TTS] FFmpeg failed with stderr: {}", stderr);
            eprintln!("[TTS] FFmpeg stdout: {}", stdout);
            return Err(TTSError::NetworkError(format!("ffmpeg failed: {}", stderr)));
        }
        
        eprintln!("[TTS] FFmpeg concatenation successful");
        
        // Read the concatenated file
        let mut buffer = Vec::new();
        std::fs::File::open(output_file.path())
            .and_then(|mut f| std::io::Read::read_to_end(&mut f, &mut buffer))
            .map_err(|e| TTSError::NetworkError(format!("Failed to read output file: {}", e)))?;
        
        eprintln!("[TTS] Successfully concatenated audio ({} bytes)", buffer.len());
        
        // Track usage for all chunks
        let _ = self.track_usage(text, voice_id, "tts-1-hd", true, None).await;
        
        Ok(buffer)
    }
    
    pub async fn generate_speech_with_model(&self, text: &str, voice_id: &str, model: &str) -> Result<Vec<u8>, TTSError> {
        const MAX_CHUNK_SIZE: usize = 4000; // Leave buffer for safety
        
        if text.len() <= MAX_CHUNK_SIZE {
            // Text fits in single request
            self.generate_speech_with_model_single(text, voice_id, model).await
        } else {
            // Use FFmpeg concatenation for long text
            eprintln!("[TTS] Text is {} characters, using FFmpeg concatenation", text.len());
            // Check if FFmpeg is available
            match Command::new("which").arg("ffmpeg").output() {
                Ok(output) if output.status.success() => {
                    eprintln!("[TTS] FFmpeg found, using concatenation");
                    self.generate_speech_with_ffmpeg_concat(text, voice_id).await
                }
                _ => {
                    eprintln!("[TTS] FFmpeg not found, using fallback single chunk");
                    // Fallback: just use the first 4000 characters with the given model
                    let truncated = if text.len() > 4000 {
                        &text[..4000]
                    } else {
                        text
                    };
                    eprintln!("[TTS] WARNING: Text truncated to {} characters", truncated.len());
                    self.generate_speech_with_model_single(truncated, voice_id, model).await
                }
            }
        }
    }
    
    async fn generate_speech_with_model_single(&self, text: &str, voice_id: &str, model: &str) -> Result<Vec<u8>, TTSError> {
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

    pub async fn generate_speech_chunked(&self, text: &str, voice_id: &str) -> Result<Vec<Vec<u8>>, TTSError> {
        const MAX_CHUNK_SIZE: usize = 3800; // Safe margin under 4096
        
        eprintln!("generate_speech_chunked called with {} characters", text.len());
        
        if text.len() <= MAX_CHUNK_SIZE {
            // Single chunk - return as single-element vector
            eprintln!("Text fits in single chunk");
            let audio = self.generate_speech_tracked_single(text, voice_id).await?;
            Ok(vec![audio])
        } else {
            // Multiple chunks needed
            let chunks = self.split_text_semantically(text, MAX_CHUNK_SIZE);
            eprintln!("Split text into {} chunks", chunks.len());
            let mut audio_chunks = Vec::new();
            
            for (i, chunk) in chunks.iter().enumerate() {
                eprintln!("Processing chunk {} of {} ({} chars)", i + 1, chunks.len(), chunk.len());
                // Add delay between API calls to avoid rate limiting
                if i > 0 {
                    sleep(Duration::from_millis(200)).await;
                }
                
                let audio = self.generate_speech_tracked_single(chunk, voice_id).await?;
                eprintln!("Chunk {} generated {} bytes of audio", i + 1, audio.len());
                audio_chunks.push(audio);
            }
            
            Ok(audio_chunks)
        }
    }
    
    async fn generate_speech_tracked_single(&self, text: &str, voice_id: &str) -> Result<Vec<u8>, TTSError> {
        let model_id = "tts-1-hd"; // OpenAI high-quality model
        
        // Generate speech for a single chunk
        match self.generate_speech(text, voice_id).await {
            Ok(audio_data) => {
                self.track_usage(text, voice_id, model_id, true, None).await?;
                Ok(audio_data)
            }
            Err(error) => {
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
    
    /// Split text into chunks at sentence boundaries when possible
    /// Based on best practices from tts-joinery and text-splitter implementations
    fn split_text_semantically(&self, text: &str, max_size: usize) -> Vec<String> {
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        
        // Split by common sentence endings
        let sentence_endings = [". ", "! ", "? ", ".\n", "!\n", "?\n"];
        let mut remaining_text = text;
        
        while !remaining_text.is_empty() {
            // Find the next sentence boundary
            let mut sentence_end = None;
            for ending in &sentence_endings {
                if let Some(pos) = remaining_text.find(ending) {
                    let end_pos = pos + ending.len();
                    if sentence_end.is_none() || end_pos < sentence_end.unwrap() {
                        sentence_end = Some(end_pos);
                    }
                }
            }
            
            let (sentence, rest) = if let Some(end_pos) = sentence_end {
                remaining_text.split_at(end_pos)
            } else {
                // No sentence boundary found, take the whole remaining text
                (remaining_text, "")
            };
            
            // Check if adding this sentence would exceed the limit
            if !current_chunk.is_empty() && current_chunk.len() + sentence.len() > max_size {
                // Save current chunk and start a new one
                chunks.push(current_chunk.clone());
                current_chunk.clear();
            }
            
            // Handle case where single sentence exceeds max_size
            if sentence.len() > max_size {
                // Split long sentence at word boundaries
                let words: Vec<&str> = sentence.split_whitespace().collect();
                for word in words {
                    if current_chunk.len() + word.len() + 1 > max_size {
                        if !current_chunk.is_empty() {
                            chunks.push(current_chunk.clone());
                            current_chunk.clear();
                        }
                    }
                    if !current_chunk.is_empty() {
                        current_chunk.push(' ');
                    }
                    current_chunk.push_str(word);
                }
            } else {
                current_chunk.push_str(sentence);
            }
            
            remaining_text = rest;
        }
        
        // Add the last chunk if not empty
        if !current_chunk.is_empty() {
            chunks.push(current_chunk);
        }
        
        chunks
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