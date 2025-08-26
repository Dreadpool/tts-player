use sqlx::{sqlite::SqlitePool, Row};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use anyhow::Result;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UsageRecord {
    pub id: Option<i64>,
    pub timestamp: DateTime<Utc>,
    pub text: String,
    pub character_count: i32,
    pub voice_id: String,
    pub model_id: String,
    pub success: bool,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub subscription_tier: String,
    pub character_limit: i32,
    pub character_used: i32,
    pub characters_remaining: i32,
    pub reset_date: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    pub total_requests: i64,
    pub total_characters: i64,
    pub successful_requests: i64,
    pub failed_requests: i64,
    pub most_used_voice: String,
    pub daily_usage: Vec<DailyUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyUsage {
    pub date: String,
    pub character_count: i64,
    pub request_count: i64,
}

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new() -> Result<Self> {
        // Create database file in app data directory  
        let app_dir = dirs::home_dir()
            .unwrap_or_else(|| std::env::temp_dir())
            .join(".tts-player");
        
        std::fs::create_dir_all(&app_dir)?;
        let db_path = app_dir.join("tts_usage.db");
        
        // Use proper SQLite URL with create flag
        let database_url = format!("sqlite://{}?mode=rwc", db_path.display());
        let pool = SqlitePool::connect(&database_url).await?;
        
        let database = Self { pool };
        database.migrate().await?;
        
        Ok(database)
    }

    async fn migrate(&self) -> Result<()> {
        // Create usage_records table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS usage_records (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                text TEXT NOT NULL,
                character_count INTEGER NOT NULL,
                voice_id TEXT NOT NULL,
                model_id TEXT NOT NULL,
                success BOOLEAN NOT NULL,
                error_message TEXT
            )
            "#
        )
        .execute(&self.pool)
        .await?;

        // Create user_info_cache table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS user_info_cache (
                id INTEGER PRIMARY KEY DEFAULT 1,
                subscription_tier TEXT,
                character_limit INTEGER,
                character_used INTEGER,
                characters_remaining INTEGER,
                last_updated DATETIME,
                reset_date DATETIME
            )
            "#
        )
        .execute(&self.pool)
        .await?;

        // Create indexes for performance
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_usage_timestamp ON usage_records(timestamp)")
            .execute(&self.pool)
            .await?;
            
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_usage_voice ON usage_records(voice_id)")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn record_usage(&self, record: &UsageRecord) -> Result<i64> {
        let id = sqlx::query(
            r#"
            INSERT INTO usage_records (timestamp, text, character_count, voice_id, model_id, success, error_message)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&record.timestamp)
        .bind(&record.text)
        .bind(record.character_count)
        .bind(&record.voice_id)
        .bind(&record.model_id)
        .bind(record.success)
        .bind(&record.error_message)
        .execute(&self.pool)
        .await?
        .last_insert_rowid();

        Ok(id)
    }

    pub async fn get_usage_records(&self, limit: i32, days: Option<i32>) -> Result<Vec<UsageRecord>> {
        let query = match days {
            Some(days) => {
                sqlx::query_as::<_, UsageRecord>(
                    r#"
                    SELECT * FROM usage_records 
                    WHERE timestamp > datetime('now', '-' || ? || ' days')
                    ORDER BY timestamp DESC 
                    LIMIT ?
                    "#
                )
                .bind(days)
                .bind(limit)
            }
            None => {
                sqlx::query_as::<_, UsageRecord>(
                    r#"
                    SELECT * FROM usage_records 
                    ORDER BY timestamp DESC 
                    LIMIT ?
                    "#
                )
                .bind(limit)
            }
        };

        let records = query.fetch_all(&self.pool).await?;
        Ok(records)
    }

    pub async fn get_usage_stats(&self, days: i32) -> Result<UsageStats> {
        // Total stats
        let total_row = sqlx::query(
            r#"
            SELECT 
                COUNT(*) as total_requests,
                SUM(character_count) as total_characters,
                SUM(CASE WHEN success THEN 1 ELSE 0 END) as successful_requests,
                SUM(CASE WHEN NOT success THEN 1 ELSE 0 END) as failed_requests
            FROM usage_records 
            WHERE timestamp > datetime('now', '-' || ? || ' days')
            "#
        )
        .bind(days)
        .fetch_one(&self.pool)
        .await?;

        let total_requests: i64 = total_row.get("total_requests");
        let total_characters: i64 = total_row.get::<Option<i64>, _>("total_characters").unwrap_or(0);
        let successful_requests: i64 = total_row.get("successful_requests");
        let failed_requests: i64 = total_row.get("failed_requests");

        // Most used voice
        let most_used_voice = sqlx::query(
            r#"
            SELECT voice_id, COUNT(*) as usage_count 
            FROM usage_records 
            WHERE timestamp > datetime('now', '-' || ? || ' days')
            GROUP BY voice_id 
            ORDER BY usage_count DESC 
            LIMIT 1
            "#
        )
        .bind(days)
        .fetch_optional(&self.pool)
        .await?
        .map(|row| row.get::<String, _>("voice_id"))
        .unwrap_or_else(|| "rachel".to_string());

        // Daily usage
        let daily_usage_rows = sqlx::query(
            r#"
            SELECT 
                date(timestamp) as date,
                SUM(character_count) as character_count,
                COUNT(*) as request_count
            FROM usage_records 
            WHERE timestamp > datetime('now', '-' || ? || ' days')
            GROUP BY date(timestamp)
            ORDER BY date DESC
            "#
        )
        .bind(days)
        .fetch_all(&self.pool)
        .await?;

        let daily_usage = daily_usage_rows
            .into_iter()
            .map(|row| DailyUsage {
                date: row.get("date"),
                character_count: row.get::<Option<i64>, _>("character_count").unwrap_or(0),
                request_count: row.get("request_count"),
            })
            .collect();

        Ok(UsageStats {
            total_requests,
            total_characters,
            successful_requests,
            failed_requests,
            most_used_voice,
            daily_usage,
        })
    }

    pub async fn cache_user_info(&self, user_info: &UserInfo) -> Result<()> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO user_info_cache 
            (id, subscription_tier, character_limit, character_used, characters_remaining, last_updated, reset_date)
            VALUES (1, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&user_info.subscription_tier)
        .bind(user_info.character_limit)
        .bind(user_info.character_used)
        .bind(user_info.characters_remaining)
        .bind(&user_info.last_updated)
        .bind(&user_info.reset_date)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_cached_user_info(&self) -> Result<Option<UserInfo>> {
        let row = sqlx::query(
            r#"
            SELECT subscription_tier, character_limit, character_used, characters_remaining, last_updated, reset_date
            FROM user_info_cache 
            WHERE id = 1
            "#
        )
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(Some(UserInfo {
                subscription_tier: row.get("subscription_tier"),
                character_limit: row.get("character_limit"),
                character_used: row.get("character_used"),
                characters_remaining: row.get("characters_remaining"),
                last_updated: row.get("last_updated"),
                reset_date: row.get("reset_date"),
            })),
            None => Ok(None),
        }
    }

    pub async fn cleanup_old_records(&self, days: i32) -> Result<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM usage_records 
            WHERE timestamp < datetime('now', '-' || ? || ' days')
            "#
        )
        .bind(days)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_creation() {
        let db = Database::new().await.unwrap();
        
        // Test recording usage
        let record = UsageRecord {
            id: None,
            timestamp: Utc::now(),
            text: "Hello world".to_string(),
            character_count: 11,
            voice_id: "rachel".to_string(),
            model_id: "eleven_multilingual_v2".to_string(),
            success: true,
            error_message: None,
        };

        let id = db.record_usage(&record).await.unwrap();
        assert!(id > 0);

        // Test retrieving usage
        let records = db.get_usage_records(10, None).await.unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].text, "Hello world");
    }

    #[tokio::test]
    async fn test_usage_stats() {
        let db = Database::new().await.unwrap();
        
        // Record some test data
        for i in 0..5 {
            let record = UsageRecord {
                id: None,
                timestamp: Utc::now(),
                text: format!("Test message {}", i),
                character_count: 10 + i,
                voice_id: if i % 2 == 0 { "rachel" } else { "adam" }.to_string(),
                model_id: "eleven_multilingual_v2".to_string(),
                success: i != 2, // Make one fail
                error_message: if i == 2 { Some("Test error".to_string()) } else { None },
            };
            db.record_usage(&record).await.unwrap();
        }

        let stats = db.get_usage_stats(7).await.unwrap();
        assert_eq!(stats.total_requests, 5);
        assert_eq!(stats.successful_requests, 4);
        assert_eq!(stats.failed_requests, 1);
        assert_eq!(stats.most_used_voice, "rachel"); // 3 uses vs 2 for adam
    }
}