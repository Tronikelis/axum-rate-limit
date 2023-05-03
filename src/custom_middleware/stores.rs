use async_trait::async_trait;
use redis::{aio::Connection, AsyncCommands, Client};
use std::{collections::HashMap, thread::current, time};

use super::main::Options;

#[derive(Debug)]
pub struct StoreError {
    msg: String,
}

#[async_trait]
pub trait Store
where
    Self: Clone,
{
    async fn get(&mut self, key: &str) -> Result<Option<usize>, StoreError>;
    async fn update(&mut self, key: &str, value: usize) -> Result<(), StoreError>;
    async fn del(&mut self, key: &str) -> Result<(), StoreError>;
}

#[derive(Clone)]
pub struct RedisStore {
    client: Client,
    options: Options,
}

impl RedisStore {
    pub fn new(client: Client, options: Options) -> Self {
        return Self { client, options };
    }

    async fn get_async_connection(&self) -> Result<Connection, StoreError> {
        return self
            .client
            .get_async_connection()
            .await
            .map_err(|_err| StoreError {
                msg: "yo".to_string(),
            });
    }
}

#[async_trait]
impl Store for RedisStore {
    async fn get(&mut self, key: &str) -> Result<Option<usize>, StoreError> {
        let mut connection = self.get_async_connection().await?;

        let value = connection.get(key).await.map_err(|_err| StoreError {
            msg: "yo".to_string(),
        })?;

        return Ok(value);
    }

    async fn update(&mut self, key: &str, value: usize) -> Result<(), StoreError> {
        let mut connection =
            self.client
                .get_async_connection()
                .await
                .map_err(|_err| StoreError {
                    msg: "yo".to_string(),
                })?;

        connection
            .set_ex(key, value, self.options.per_min * 60)
            .await
            .map_err(|_err| StoreError {
                msg: "yo".to_string(),
            })?;

        return Ok(());
    }

    async fn del(&mut self, key: &str) -> Result<(), StoreError> {
        let mut connection =
            self.client
                .get_async_connection()
                .await
                .map_err(|_err| StoreError {
                    msg: "yo".to_string(),
                })?;

        connection.del(key).await.map_err(|_err| StoreError {
            msg: "yo".to_string(),
        })?;

        return Ok(());
    }
}

#[derive(Clone)]
struct MemoryValue {
    value: usize,
    updated_at: usize,
}

#[derive(Clone)]
pub struct MemoryStore {
    hash_map: HashMap<String, MemoryValue>,
    options: Options,
}

impl MemoryStore {
    pub fn new(options: Options) -> Self {
        return Self {
            hash_map: HashMap::new(),
            options,
        };
    }
}

#[async_trait]
impl Store for MemoryStore {
    async fn get(&mut self, key: &str) -> Result<Option<usize>, StoreError> {
        let current_unix = time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let got = self.hash_map.get(key);

        return match got {
            Some(got) => {
                if got.updated_at + (self.options.per_min * 60) < current_unix.try_into().unwrap() {
                    self.del(key).await?;
                    Ok(None)
                } else {
                    Ok(Some(got.value))
                }
            }
            None => Ok(None),
        };
    }

    async fn del(&mut self, key: &str) -> Result<(), StoreError> {
        self.hash_map.remove(key);
        return Ok(());
    }

    async fn update(&mut self, key: &str, value: usize) -> Result<(), StoreError> {
        let current_unix = time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let current = self.hash_map.get(key);

        match current {
            Some(current) => self.hash_map.insert(
                key.to_string(),
                MemoryValue {
                    value,
                    updated_at: current.updated_at,
                },
            ),
            None => self.hash_map.insert(
                key.to_string(),
                MemoryValue {
                    value,
                    updated_at: current_unix.try_into().unwrap(),
                },
            ),
        };

        return Ok(());
    }
}
