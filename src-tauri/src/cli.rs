use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CliArgs {
    pub text: Option<String>,
    pub voice: Option<String>,
}

pub fn parse_cli_args(args: Vec<String>) -> Result<CliArgs, String> {
    let mut cli_args = CliArgs {
        text: None,
        voice: None,
    };
    
    let mut i = 1; // Skip program name
    while i < args.len() {
        match args[i].as_str() {
            "--text" | "-t" => {
                if i + 1 < args.len() {
                    cli_args.text = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --text argument".to_string());
                }
            }
            "--voice" | "-v" => {
                if i + 1 < args.len() {
                    cli_args.voice = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --voice argument".to_string());
                }
            }
            "--help" | "-h" => {
                return Err(format_help());
            }
            _ => {
                return Err(format!("Unknown argument: {}", args[i]));
            }
        }
    }
    
    Ok(cli_args)
}

fn format_help() -> String {
    r#"TTS Player - Text-to-Speech Audio Player

USAGE:
    tts-player [OPTIONS]

OPTIONS:
    -t, --text <TEXT>     Text to convert to speech
    -v, --voice <VOICE>   Voice ID to use (rachel, adam, bella)
    -h, --help           Print help information

EXAMPLES:
    tts-player --text "Hello world"
    tts-player -t "Hello world" -v rachel
"#.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_missing_text_value() {
        let args = vec!["app".to_string(), "--text".to_string()];
        let result = parse_cli_args(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_help_flag() {
        let args = vec!["app".to_string(), "--help".to_string()];
        let result = parse_cli_args(args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("USAGE"));
    }
}