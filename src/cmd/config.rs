use crate::{app, Result};
use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    #[serde(rename = "App", default)]
    pub app: app::Config,
}

impl Config {
    pub fn validate(&self) -> Result<()> {
        self.app.validate()?;
        Ok(())
    }

    pub async fn load_from_env(&mut self, prefix: &str) -> Result<()> {
        self.app.load_from_env([prefix, "APP_"].join("").as_str()).await?;
        Ok(())
    }
}
