#[cfg(test)]
mod tts_service_tests {
    use mockito::{Matcher, Server};
    use crate::tts::{TTSService, TTSError};
    use tokio_test;

    #[tokio::test]
    async fn test_successful_tts_generation() {
        let mut server = Server::new_async().await;
        
        let mock = server
            .mock("POST", "/v1/text-to-speech/rachel")
            .match_header("xi-api-key", "test-api-key")
            .match_body(Matcher::JsonString(r#"{"text":"Hello world","model_id":"eleven_multilingual_v2"}"#.to_string()))
            .with_status(200)
            .with_header("Content-Type", "audio/mpeg")
            .with_body(vec![1, 2, 3, 4]) // Mock audio data
            .create_async()
            .await;

        let service = TTSService::new("test-api-key", &server.url());
        let result = service.generate_speech("Hello world", "rachel").await;
        
        assert!(result.is_ok());
        let audio_data = result.unwrap();
        assert_eq!(audio_data, vec![1, 2, 3, 4]);
        
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_missing_api_key() {
        let mut server = Server::new_async().await;
        
        let mock = server
            .mock("POST", "/v1/text-to-speech/rachel")
            .with_status(401)
            .with_body(r#"{"detail":"Missing API key"}"#)
            .create_async()
            .await;

        let service = TTSService::new("", &server.url());
        let result = service.generate_speech("Hello world", "rachel").await;
        
        assert!(result.is_err());
        match result.unwrap_err() {
            TTSError::Authentication(msg) => assert!(msg.contains("Missing API key")),
            _ => panic!("Expected authentication error"),
        }
        
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let mut server = Server::new_async().await;
        
        let mock = server
            .mock("POST", "/v1/text-to-speech/rachel")
            .with_status(429)
            .with_header("Retry-After", "60")
            .with_body(r#"{"detail":"Rate limit exceeded"}"#)
            .create_async()
            .await;

        let service = TTSService::new("test-api-key", &server.url());
        let result = service.generate_speech("Hello world", "rachel").await;
        
        assert!(result.is_err());
        match result.unwrap_err() {
            TTSError::RateLimit(retry_after) => assert_eq!(retry_after, Some(60)),
            _ => panic!("Expected rate limit error"),
        }
        
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_retry_logic() {
        let mut server = Server::new_async().await;
        let mut call_count = 0;
        
        // Mock first two calls to fail, third to succeed
        let mock1 = server
            .mock("POST", "/v1/text-to-speech/rachel")
            .with_status(500)
            .with_body("Internal server error")
            .expect(1)
            .create_async()
            .await;
            
        let mock2 = server
            .mock("POST", "/v1/text-to-speech/rachel")
            .with_status(500)
            .with_body("Internal server error")
            .expect(1)
            .create_async()
            .await;
            
        let mock3 = server
            .mock("POST", "/v1/text-to-speech/rachel")
            .with_status(200)
            .with_header("Content-Type", "audio/mpeg")
            .with_body(vec![1, 2, 3, 4])
            .expect(1)
            .create_async()
            .await;

        let service = TTSService::new("test-api-key", &server.url());
        let result = service.generate_speech_with_retry("Hello world", "rachel").await;
        
        assert!(result.is_ok());
        
        mock1.assert_async().await;
        mock2.assert_async().await;
        mock3.assert_async().await;
    }

    #[tokio::test]
    async fn test_voice_validation() {
        let service = TTSService::new("test-api-key", "https://api.elevenlabs.io");
        
        let valid_voices = vec!["rachel", "adam", "bella"];
        for voice in valid_voices {
            assert!(service.is_valid_voice(voice));
        }
        
        assert!(!service.is_valid_voice("invalid-voice"));
        assert!(!service.is_valid_voice(""));
    }

    #[tokio::test]
    async fn test_text_length_validation() {
        let service = TTSService::new("test-api-key", "https://api.elevenlabs.io");
        
        // Test empty text
        let result = service.validate_text("").await;
        assert!(result.is_err());
        
        // Test very long text (over 5000 chars)
        let long_text = "a".repeat(5001);
        let result = service.validate_text(&long_text).await;
        assert!(result.is_err());
        
        // Test valid text
        let result = service.validate_text("Hello world").await;
        assert!(result.is_ok());
    }
}