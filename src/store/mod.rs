use crate::store_key;
use crate::{model, Error as BoxError};
use chrono::{DateTime, Utc};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use keyvaluestore::{dynamodbstore, dynstore, memorystore, readcache, redisstore, Arg, Backend, BatchOperation};
use serde::de::Deserialize;
use serde::ser::Serialize;
use simple_error::SimpleError;
use std::io::{Read, Write};

pub mod config;
pub use config::*;
pub mod error;
pub use error::*;
pub mod conditionals;
pub mod store_key;
pub use store_key::*;
pub mod client;
pub use client::*;
pub mod transaction;
pub use transaction::*;
pub mod dispute;
pub use dispute::*;

#[derive(Clone)]
pub struct Store<B> {
    backend: B,
}

impl<B: Backend + Sync> Store<B> {
    pub fn time_microsecond_score(t: &DateTime<Utc>) -> f64 {
        t.timestamp() as f64 * 1000000.0 + t.timestamp_subsec_micros() as f64
    }

    /// Gets members that have been inserted into a sorted set or hash.
    async fn get_by_score<'a, K: Into<Arg<'a>> + Send, T: for<'de> Deserialize<'de>>(
        &self,
        key: K,
        min: f64,
        max: f64,
        limit: i32,
        type_key: &str,
    ) -> Result<Vec<T>> {
        let values = if limit < 0 {
            self.backend.zh_rev_range_by_score(key, min, max, -limit as _).await?
        } else {
            self.backend.zh_range_by_score(key, min, max, limit as _).await?
        };

        let mut members: Vec<Option<T>> = Vec::with_capacity(values.len());
        let mut ids: Vec<model::Id> = vec![];
        let mut id_indices = vec![];
        for v in values.into_iter() {
            match v.as_ref().len() {
                model::ID_LENGTH => {
                    ids.push(v.to_vec().into());
                    id_indices.push(members.len());
                    members.push(None);
                }
                _ => members.push(Self::deserialize(v.as_ref())?),
            }
        }

        let mut batch = BatchOperation::new();
        let gets: Vec<_> = ids.into_iter().map(|id| batch.get(store_key!(type_key, ":", id))).collect();
        self.backend.exec_batch(batch).await?;
        for (i, get) in gets.into_iter().enumerate() {
            members[id_indices[i]] = get.value().map(|v| -> Result<T> { Self::deserialize(v.as_ref()) }).transpose()?;
        }
        Ok(members.into_iter().flatten().collect())
    }

    /// Gets members that have been inserted into a sorted set or hash using their time as their score.
    async fn get_by_time_range<'a, K: Into<Arg<'a>> + Send, T: for<'de> Deserialize<'de>>(
        &self,
        key: K,
        min: DateTime<Utc>,
        max: DateTime<Utc>,
        limit: i32,
        type_key: &str,
    ) -> Result<Vec<T>> {
        self.get_by_score(key, Self::time_microsecond_score(&min), Self::time_microsecond_score(&max), limit, type_key)
            .await
    }

    pub fn serialize<T: Serialize>(v: &T) -> Result<Vec<u8>> {
        let buf = rmp_serde::to_vec_named(v)?;
        let mut e = GzEncoder::new(Vec::new(), Compression::default());
        e.write_all(&buf)?;
        Ok(e.finish()?)
    }

    pub fn deserialize<T: for<'de> Deserialize<'de>>(buf: &[u8]) -> Result<T> {
        let mut gz = GzDecoder::new(buf);
        let mut buf = Vec::new();
        gz.read_to_end(&mut buf)?;
        Ok(rmp_serde::from_slice(&buf)?)
    }
}

impl Store<dynstore::Backend> {
    #[cfg(test)]
    pub fn new_test_store() -> Self {
        let config = Config {
            in_memory: true,
            ..Default::default()
        };
        Store::new_with_config(&config).unwrap()
    }

    pub fn new_with_config(config: &Config) -> Result<Self> {
        let backend = if config.in_memory {
            dynstore::Backend::Memory(memorystore::Backend::new())
        } else if let Some(addr) = &config.redis_address {
            println!("REDIS");
            dynstore::Backend::Redis(redisstore::Backend::new(redis::Client::open(("redis://".to_string() + addr).as_str())?))
        } else if let Some(config) = &config.dynamodb {
            use keyvaluestore::rusoto_core::{region::Region, request::HttpClient};
            use keyvaluestore::rusoto_credential::DefaultCredentialsProvider;
            use keyvaluestore::rusoto_dynamodb::DynamoDbClient;

            let client = match &config.endpoint {
                Some(endpoint) => DynamoDbClient::new_with(
                    HttpClient::new().map_err(|e| -> BoxError { Box::new(e) })?,
                    DefaultCredentialsProvider::new().map_err(|e| -> BoxError { Box::new(e) })?,
                    Region::Custom {
                        name: "custom".to_string(),
                        endpoint: endpoint.clone(),
                    },
                ),
                None => DynamoDbClient::new(Region::default()),
            };
            dynstore::Backend::DynamoDB(dynamodbstore::Backend {
                allow_eventually_consistent_reads: false,
                client,
                table_name: config.table_name.clone(),
            })
        } else {
            return Err(Error::Other(Box::new(SimpleError::new("invalid store config"))));
        };
        Ok(Store { backend })
    }

    fn backend_with_eventually_consistent_reads(b: dynstore::Backend) -> dynstore::Backend {
        match b {
            dynstore::Backend::DynamoDB(b) => dynstore::Backend::DynamoDB(dynamodbstore::Backend {
                allow_eventually_consistent_reads: true,
                client: b.client.clone(),
                table_name: b.table_name,
            }),
            dynstore::Backend::ReadCache(b) => dynstore::Backend::ReadCache(Box::new(readcache::Backend::new(Self::backend_with_eventually_consistent_reads(
                b.into_inner(),
            )))),
            b => b,
        }
    }

    pub fn with_eventually_consistent_reads(&self) -> Store<dynstore::Backend> {
        Store {
            backend: Self::backend_with_eventually_consistent_reads(self.backend.clone()),
        }
    }

    fn backend_with_read_cache(b: dynstore::Backend) -> dynstore::Backend {
        match b {
            dynstore::Backend::ReadCache(_) => b,
            b => dynstore::Backend::ReadCache(Box::new(readcache::Backend::new(b))),
        }
    }

    pub fn with_read_cache(&self) -> Store<dynstore::Backend> {
        Store {
            backend: Self::backend_with_read_cache(self.backend.clone()),
        }
    }
}
