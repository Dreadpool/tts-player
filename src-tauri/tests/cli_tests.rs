#[cfg(test)]
mod cli_tests {
    use crate::cli::{parse_cli_args, CliArgs};

    #[test]
    fn test_text_parameter_parsing() {
        let args = vec!["app".to_string(), "--text".to_string(), "Hello world".to_string()];
        let parsed = parse_cli_args(args).unwrap();
        assert_eq!(parsed.text, Some("Hello world".to_string()));
    }

    #[test]
    fn test_voice_parameter_parsing() {
        let args = vec![
            "app".to_string(),
            "--text".to_string(),
            "Hello".to_string(),
            "--voice".to_string(),
            "rachel".to_string(),
        ];
        let parsed = parse_cli_args(args).unwrap();
        assert_eq!(parsed.text, Some("Hello".to_string()));
        assert_eq!(parsed.voice, Some("rachel".to_string()));
    }

    #[test]
    fn test_url_encoded_text() {
        let args = vec!["app".to_string(), "--text".to_string(), "Hello%20world%21".to_string()];
        let parsed = parse_cli_args(args).unwrap();
        let decoded = urlencoding::decode(&parsed.text.unwrap()).unwrap();
        assert_eq!(decoded, "Hello world!");
    }

    #[test]
    fn test_short_flags() {
        let args = vec!["app".to_string(), "-t".to_string(), "Hello".to_string()];
        let parsed = parse_cli_args(args).unwrap();
        assert_eq!(parsed.text, Some("Hello".to_string()));
    }

    #[test]
    fn test_empty_args() {
        let args = vec!["app".to_string()];
        let parsed = parse_cli_args(args).unwrap();
        assert_eq!(parsed.text, None);
        assert_eq!(parsed.voice, None);
    }

    #[test]
    fn test_special_characters() {
        let special_text = "Hello! 你好! Émojis 🎵";
        let args = vec!["app".to_string(), "--text".to_string(), special_text.to_string()];
        let parsed = parse_cli_args(args).unwrap();
        assert_eq!(parsed.text, Some(special_text.to_string()));
    }
}