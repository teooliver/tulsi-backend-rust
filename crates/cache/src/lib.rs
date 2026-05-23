use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use tracing::warn;

const DEFAULT_TTL_SECS: u64 = 60;

#[derive(Clone)]
pub struct RedisCache {
    conn: ConnectionManager,
}

impl RedisCache {
    pub fn new(conn: ConnectionManager) -> Self {
        Self { conn }
    }

    pub async fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        let mut conn = self.conn.clone();
        match conn.get::<_, Option<String>>(key).await {
            Ok(Some(data)) => match serde_json::from_str(&data) {
                Ok(val) => Some(val),
                Err(e) => {
                    warn!(key, error = %e, "Failed to deserialize cached value");
                    None
                }
            },
            Ok(None) => None,
            Err(e) => {
                warn!(key, error = %e, "Redis GET failed");
                None
            }
        }
    }

    pub async fn set<T: Serialize>(&self, key: &str, value: &T) {
        let mut conn = self.conn.clone();
        match serde_json::to_string(value) {
            Ok(data) => {
                if let Err(e) = conn
                    .set_ex::<_, _, ()>(key, &data, DEFAULT_TTL_SECS)
                    .await
                {
                    warn!(key, error = %e, "Redis SET failed");
                }
            }
            Err(e) => {
                warn!(key, error = %e, "Failed to serialize value for cache");
            }
        }
    }

    pub async fn delete(&self, keys: &[&str]) {
        let mut conn = self.conn.clone();
        for key in keys {
            if let Err(e) = conn.del::<_, ()>(*key).await {
                warn!(key, error = %e, "Redis DEL failed");
            }
        }
    }
}
