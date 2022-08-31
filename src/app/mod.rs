use crate::{store, Result};
use chrono::{NaiveDateTime, Utc};
use keyvaluestore::dynstore;

pub mod config;
pub use config::*;
pub mod error;
pub use error::*;
pub mod session;
pub use session::*;
pub mod transaction;
pub use transaction::*;
pub mod process_transaction;
pub use process_transaction::*;
pub mod client;
pub use client::*;
pub mod dispute;
pub use dispute::*;

pub struct App {
    config: Config,
    store: store::Store<dynstore::Backend>,
}

impl App {
    pub async fn new_with_config(config: Config) -> Result<App> {
        let store = store::Store::new_with_config(&config.store)?;

        Ok(Self { store, config })
    }

    pub fn new_session(&self, logger: slog::Logger) -> Session {
        Session {
            app: self,
            store: self.store.clone(),
            inner_logger: logger,
        }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }
}

#[cfg(test)]
impl App {
    pub async fn new_test_app<F: FnOnce(&mut Config)>(configure: F) -> Self {
        let mut config = Config {
            store: store::Config {
                in_memory: true,
                ..Default::default()
            },
            ..Default::default()
        };
        configure(&mut config);
        Self::new_with_config(config).await.expect("failed to create test app")
    }

    pub fn test_logger() -> slog::Logger {
        use slog::Drain;
        let decorator = slog_term::TermDecorator::new().build();
        let drain = slog_term::FullFormat::new(decorator).build().fuse();
        let drain = slog_async::Async::new(drain).build().filter_level(slog::Level::Warning).fuse();
        slog::Logger::root(drain, o!())
    }
}

pub fn distant_past() -> chrono::DateTime<Utc> {
    chrono::DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(-2208988800, 0), Utc)
}

pub fn distant_future() -> chrono::DateTime<Utc> {
    chrono::DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(7258118400, 0), Utc)
}
