use super::Cache;

use std::path::PathBuf;

pub struct FilesystemCache {
    dir: PathBuf,
}

impl FilesystemCache {
    pub fn tmp() -> Self {
        Self {
            dir: std::env::temp_dir().join("aoc-cache"),
        }
    }

    pub fn clean_tmp() -> std::io::Result<Self> {
        let cache = Self::tmp();
        std::fs::remove_dir_all(&cache.dir)?;
        Ok(cache)
    }
}

impl Cache for FilesystemCache {
    async fn cache(&mut self, key: &str, value: &str) {
        let _ = std::fs::create_dir(&self.dir);
        tokio::fs::write(self.dir.join(key), value)
            .await
            .expect("Failed to write to cache");
    }

    async fn recall(&self, key: &str) -> Option<String> {
        tokio::fs::read_to_string(self.dir.join(key)).await.ok()
    }

    async fn dirty(&mut self, key: &str) {
        let _ = tokio::fs::remove_file(self.dir.join(key)).await;
    }
}
