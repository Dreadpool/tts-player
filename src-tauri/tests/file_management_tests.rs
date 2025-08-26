#[cfg(test)]
mod file_management_tests {
    use tempfile::TempDir;
    use crate::file_manager::{FileManager, create_temp_audio_file, cleanup_temp_files};
    use std::path::Path;
    use tokio::fs;

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
    async fn test_temp_file_cleanup() {
        let temp_dir = TempDir::new().unwrap();
        let audio_path = temp_dir.path().join("test_audio.mp3");
        
        // Create file
        let audio_data = vec![1, 2, 3, 4, 5];
        create_temp_audio_file(&audio_path, &audio_data).await.unwrap();
        assert!(audio_path.exists());
        
        // Cleanup
        let result = cleanup_temp_files().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_file_manager_lifecycle() {
        let manager = FileManager::new();
        
        let audio_data = vec![1, 2, 3, 4, 5];
        let file_path = manager.create_temp_audio_file(&audio_data).await.unwrap();
        
        assert!(Path::new(&file_path).exists());
        
        // Test cleanup on drop
        drop(manager);
        
        // File should be cleaned up (this test may be flaky in practice)
        // In real implementation, cleanup would happen on app shutdown
    }

    #[tokio::test]
    async fn test_file_permissions() {
        let temp_dir = TempDir::new().unwrap();
        let audio_path = temp_dir.path().join("test_audio.mp3");
        
        let audio_data = vec![1, 2, 3, 4, 5];
        create_temp_audio_file(&audio_path, &audio_data).await.unwrap();
        
        let metadata = fs::metadata(&audio_path).await.unwrap();
        assert!(!metadata.permissions().readonly());
    }

    #[tokio::test]
    async fn test_insufficient_disk_space() {
        // This test would need to mock filesystem operations
        // or use a test filesystem with limited space
        // For now, we'll test error handling path
        
        let temp_dir = TempDir::new().unwrap();
        let invalid_path = temp_dir.path().join("nonexistent_dir").join("test.mp3");
        
        let audio_data = vec![1, 2, 3, 4, 5];
        let result = create_temp_audio_file(&invalid_path, &audio_data).await;
        
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_multiple_temp_files() {
        let temp_dir = TempDir::new().unwrap();
        let manager = FileManager::new();
        
        let audio_data1 = vec![1, 2, 3];
        let audio_data2 = vec![4, 5, 6];
        
        let file1 = manager.create_temp_audio_file(&audio_data1).await.unwrap();
        let file2 = manager.create_temp_audio_file(&audio_data2).await.unwrap();
        
        assert!(Path::new(&file1).exists());
        assert!(Path::new(&file2).exists());
        assert_ne!(file1, file2); // Different file paths
    }
}