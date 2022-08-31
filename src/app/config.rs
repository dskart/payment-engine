use crate::store;
use crate::Result;
use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct Config {
    pub store: store::Config,
}

impl Config {
    pub fn validate(&self) -> Result<()> {
        self.store.validate()?;
        Ok(())
    }

    pub async fn load_from_env(&mut self, prefix: &str) -> Result<()> {
        self.store.load_from_env([prefix, "STORE_"].join("").as_str())?;
        Ok(())
    }
}
