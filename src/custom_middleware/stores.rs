use std::collections::HashMap;

use std::sync::Arc;

use redis::{aio::Connection, AsyncCommands, Client, Commands, RedisError};

use async_trait::async_trait;
use tokio::sync::RwLock;

#[derive(Debug)]
pub struct StoreError {
    msg: String,
}

#[async_trait]
pub trait Store
where
    Self: Clone,
{
    async fn get(&self, key: &str) -> Result<Option<usize>, StoreError>;
    async fn update(&mut self, key: &str, value: usize) -> Result<(), StoreError>;
    async fn del(&mut self, key: &str) -> Result<(), StoreError>;
}

#[derive(Clone)]
pub struct RedisStore {
    pub client: Client,
}

impl RedisStore {
    pub fn new(client: Client) -> Self {
        return Self { client };
    }

    async fn get_async_connection(&self) -> Result<Connection, StoreError> {
        return Ok(self
            .client
            .get_async_connection()
            .await
            .map_err(|err| StoreError {
                msg: "yo".to_string(),
            })?);
    }
}

#[async_trait]
impl Store for RedisStore {
    async fn get(&self, key: &str) -> Result<Option<usize>, StoreError> {
        let mut connection = self.get_async_connection().await?;

        let value = connection.get(key).await.map_err(|err| StoreError {
            msg: "yo".to_string(),
        })?;

        return Ok(value);
    }

    async fn update(&mut self, key: &str, value: usize) -> Result<(), StoreError> {
        let mut connection =
            self.client
                .get_async_connection()
                .await
                .map_err(|err| StoreError {
                    msg: "yo".to_string(),
                })?;

        connection.set(key, value).await.map_err(|err| StoreError {
            msg: "yo".to_string(),
        })?;

        return Ok(());
    }

    async fn del(&mut self, key: &str) -> Result<(), StoreError> {
        let mut connection =
            self.client
                .get_async_connection()
                .await
                .map_err(|err| StoreError {
                    msg: "yo".to_string(),
                })?;

        connection.del(key).await.map_err(|err| StoreError {
            msg: "yo".to_string(),
        })?;

        return Ok(());
    }
}

#[derive(Clone)]
pub struct MemoryStore {
    pub hash_map: HashMap<String, usize>,
}

impl MemoryStore {
    pub fn new() -> Self {
        return Self {
            hash_map: HashMap::new(),
        };
    }
}

#[async_trait]
impl Store for MemoryStore {
    async fn get(&self, key: &str) -> Result<Option<usize>, StoreError> {
        return Ok(self.hash_map.get(key).copied());
    }

    async fn del(&mut self, key: &str) -> Result<(), StoreError> {
        self.hash_map.remove(key);
        return Ok(());
    }

    async fn update(&mut self, key: &str, value: usize) -> Result<(), StoreError> {
        self.hash_map.insert(key.to_string(), value);
        return Ok(());
    }
}
