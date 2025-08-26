use std::path::{Path, PathBuf};
use tokio::fs;
use uuid::Uuid;
use anyhow::Result;

pub struct FileManager {
    temp_dir: PathBuf,
}

impl FileManager {
    pub fn new() -> Self {
        let temp_dir = std::env::temp_dir().join("tts-player");
        Self { temp_dir }
    }

    pub async fn create_temp_audio_file(&self, audio_data: &[u8]) -> Result<String> {
        // Ensure temp directory exists
        fs::create_dir_all(&self.temp_dir).await?;
        
        // Generate unique filename
        let filename = format!("{}.mp3", Uuid::new_v4());
        let file_path = self.temp_dir.join(filename);
        
        // Write audio data to file
        fs::write(&file_path, audio_data).await?;
        
        // Return absolute path as string
        Ok(file_path.to_string_lossy().to_string())
    }
}

impl Drop for FileManager {
    fn drop(&mut self) {
        // In a real implementation, we'd queue cleanup for the background
        // For now, files will be cleaned up by the OS or manual cleanup
    }
}

pub async fn create_temp_audio_file(path: &Path, audio_data: &[u8]) -> Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }
    
    // Write audio data
    fs::write(path, audio_data).await?;
    
    Ok(())
}

pub async fn cleanup_temp_files() -> Result<()> {
    let temp_dir = std::env::temp_dir().join("tts-player");
    
    if temp_dir.exists() {
        // Remove all files in temp directory
        let mut entries = fs::read_dir(&temp_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() {
                if let Err(e) = fs::remove_file(&path).await {
                    eprintln!("Failed to remove temp file {:?}: {}", path, e);
                }
            }
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_temp_file_creation() {
        let temp_dir = TempDir::new().unwrap();
        let audio_path = temp_dir.path().join("test_audio.mp3");
        
        let audio_data = vec![1, 2, 3, 4, 5];
        let result = create_temp_audio_file(&audio_path, &audio_data).await;
        
        assert!(result.is_ok());
        assert!(audio_path.exists());
        
        let written_data = fs::read(&audio_path).await.unwrap();
        assert_eq!(written_data, audio_data);
    }

    #[tokio::test]
    async fn test_file_manager_lifecycle() {
        let manager = FileManager::new();
        
        let audio_data = vec![1, 2, 3, 4, 5];
        let file_path = manager.create_temp_audio_file(&audio_data).await.unwrap();
        
        assert!(Path::new(&file_path).exists());
    }

    #[tokio::test]
    async fn test_invalid_path() {
        let temp_dir = TempDir::new().unwrap();
        let invalid_path = temp_dir.path().join("nonexistent_dir").join("test.mp3");
        
        let audio_data = vec![1, 2, 3, 4, 5];
        let result = create_temp_audio_file(&invalid_path, &audio_data).await;
        
        // Should succeed because we create parent directories
        assert!(result.is_ok());
    }
}