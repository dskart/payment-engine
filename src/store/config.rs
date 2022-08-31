use crate::store;
use serde::Deserialize;
use simple_error::SimpleError;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct Config {
    // If given, an in-memory store will be used. No data will persist after the process exits, and
    // CLI commands cannot be used to populate the store. This is really only useful for tests.
    pub in_memory: bool,

    // If given, Redis will be used for the store. This should be an IP address and port such as
    // "127.0.0.1:6379".
    pub redis_address: Option<String>,

    // If given, DynamoDB will be used for the store. Credentials are assumed to be provided via
    // the usual environment variables or an IAM role.
    #[serde(rename = "DynamoDB")]
    pub dynamodb: Option<DynamoDBConfig>,
}

impl Config {
    pub fn validate(&self) -> store::Result<()> {
        match (self.in_memory, &self.redis_address, &self.dynamodb) {
            (true, None, None) | (false, Some(_), None) | (false, None, Some(_)) => {}
            _ => {
                return Err(store::Error::Other(Box::new(SimpleError::new(
                    "exactly one type of store should be configured",
                ))))
            }
        }
        Ok(())
    }

    pub fn load_from_env(&mut self, prefix: &str) -> store::Result<()> {
        if let Ok(in_memory) = std::env::var([prefix, "INMEMORY"].join("").as_str()) {
            self.in_memory = in_memory.parse().unwrap_or(false);
        }
        if let Ok(redis_address) = std::env::var([prefix, "REDISADDRESS"].join("").as_str()) {
            self.redis_address = Some(redis_address);
        }

        if let Some(dynamodb_env) = DynamoDBConfig::load_from_env([prefix, "DYNAMODB_"].join("").as_str()) {
            self.dynamodb = Some(dynamodb_env);
        }

        Ok(())
    }
}

#[derive(Clone, Default, Debug, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct DynamoDBConfig {
    // The DynamoDB API endpoint. In development, it may be useful to run DynamoDB locally and set
    // this to http://127.0.0.1:8000.
    pub endpoint: Option<String>,

    pub table_name: String,
}

impl DynamoDBConfig {
    pub fn load_from_env(prefix: &str) -> Option<Self> {
        let mut env_cfg = Self::default();

        if let Ok(table_name) = std::env::var([prefix, "TABLENAME"].join("").as_str()) {
            env_cfg.table_name = table_name;
        }

        if let Ok(endpoint) = std::env::var([prefix, "ENDPOINT"].join("").as_str()) {
            env_cfg.endpoint = Some(endpoint);
        }

        if !env_cfg.table_name.is_empty() || env_cfg.endpoint.is_some() {
            Some(env_cfg)
        } else {
            None
        }
    }
}
