// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cli;
mod tts;
mod file_manager;
mod database;

use tauri::Manager;
use tauri_plugin_clipboard_manager::ClipboardExt;

#[tauri::command]
async fn generate_speech(text: String, voice_id: String) -> Result<String, String> {
    let api_key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| "OPENAI_API_KEY environment variable not set".to_string())?;
    
    let tts_service = tts::TTSService::with_database(&api_key, "https://api.openai.com")
        .await
        .map_err(|e| e.to_string())?;
    
    // Validate inputs
    tts_service.validate_text(&text).await?;
    if !tts_service.is_valid_voice(&voice_id) {
        return Err(format!("Invalid voice ID: {}", voice_id));
    }
    
    // Generate speech with usage tracking
    let audio_data = tts_service.generate_speech_tracked(&text, &voice_id).await?;
    
    // Convert audio data to base64 data URL that the HTML audio player can use directly
    use base64::{Engine, engine::general_purpose};
    let base64_audio = general_purpose::STANDARD.encode(&audio_data);
    let data_url = format!("data:audio/mpeg;base64,{}", base64_audio);
    
    Ok(data_url)
}

#[tauri::command]
async fn generate_speech_with_model(text: String, voice_id: String, model: String) -> Result<String, String> {
    let api_key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| "OPENAI_API_KEY environment variable not set".to_string())?;
    
    let tts_service = tts::TTSService::with_database(&api_key, "https://api.openai.com")
        .await
        .map_err(|e| e.to_string())?;
    
    // Validate inputs
    tts_service.validate_text(&text).await?;
    if !tts_service.is_valid_voice(&voice_id) {
        return Err(format!("Invalid voice ID: {}", voice_id));
    }
    
    // Generate speech with specific model
    let audio_data = tts_service.generate_speech_with_model(&text, &voice_id, &model).await?;
    
    // Track usage
    let _ = tts_service.track_usage(&text, &voice_id, &model, true, None).await;
    
    // Convert audio data to base64 data URL that the HTML audio player can use directly
    use base64::{Engine, engine::general_purpose};
    let base64_audio = general_purpose::STANDARD.encode(&audio_data);
    let data_url = format!("data:audio/mpeg;base64,{}", base64_audio);
    
    Ok(data_url)
}

#[tauri::command]
async fn get_user_info() -> Result<database::UserInfo, String> {
    let api_key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| "OPENAI_API_KEY environment variable not set".to_string())?;
    
    let tts_service = tts::TTSService::with_database(&api_key, "https://api.openai.com")
        .await
        .map_err(|e| e.to_string())?;
    
    tts_service.get_user_info().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_usage_stats(days: i32) -> Result<database::UsageStats, String> {
    let api_key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| "OPENAI_API_KEY environment variable not set".to_string())?;
    
    let tts_service = tts::TTSService::with_database(&api_key, "https://api.openai.com")
        .await
        .map_err(|e| e.to_string())?;
    
    tts_service.get_usage_stats(days).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_usage_history(limit: i32, days: Option<i32>) -> Result<Vec<database::UsageRecord>, String> {
    let api_key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| "OPENAI_API_KEY environment variable not set".to_string())?;
    
    let tts_service = tts::TTSService::with_database(&api_key, "https://api.openai.com")
        .await
        .map_err(|e| e.to_string())?;
    
    tts_service.get_usage_history(limit, days).await.map_err(|e| e.to_string())
}

#[tauri::command]
fn count_characters(text: String) -> i32 {
    text.len() as i32
}

#[tauri::command]
async fn read_clipboard(app_handle: tauri::AppHandle) -> Result<String, String> {
    // Use Tauri's clipboard API
    app_handle
        .clipboard()
        .read_text()
        .map_err(|e| format!("Failed to read clipboard: {}", e))
}

#[tauri::command]
async fn read_text_file(file_path: String) -> Result<String, String> {
    use std::fs;
    use std::path::Path;
    
    // Secure path validation using canonical paths to prevent path traversal
    let temp_dir = std::env::temp_dir();
    let temp_dir_canonical = match temp_dir.canonicalize() {
        Ok(path) => path,
        Err(_) => return Err("Unable to determine temp directory".to_string()),
    };
    
    let file_path_buf = Path::new(&file_path);
    let file_path_canonical = match file_path_buf.canonicalize() {
        Ok(path) => path,
        Err(_) => return Err("Invalid file path".to_string()),
    };
    
    // Ensure the canonical path is within the temp directory
    if !file_path_canonical.starts_with(&temp_dir_canonical) {
        return Err("Access denied: can only read from temporary directory".to_string());
    }
    
    match fs::read_to_string(&file_path) {
        Ok(content) => {
            // Clean up the file after reading
            let _ = fs::remove_file(&file_path);
            Ok(content)
        }
        Err(e) => Err(format!("Failed to read file: {}", e))
    }
}

#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_cli::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
            generate_speech,
            generate_speech_with_model,
            get_user_info,
            get_usage_stats,
            get_usage_history,
            count_characters,
            read_text_file,
            read_clipboard
        ])
        .setup(|app| {
            // Setup cleanup on app exit
            let app_handle = app.handle().clone();
            
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Regular);
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}