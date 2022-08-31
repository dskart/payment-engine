use crate::{
    app::{
        error::{self, Sanitizable, SanitizedResult},
        App,
    },
    store,
};
use keyvaluestore::dynstore;
pub struct Session<'a> {
    pub app: &'a App,
    pub store: store::Store<dynstore::Backend>,
    pub inner_logger: slog::Logger,
}

impl<'a> Session<'a> {
    pub fn logger(&self) -> slog::Logger {
        self.inner_logger.clone()
    }

    pub fn sanitize<T, E: Sanitizable>(&self, r: std::result::Result<T, E>) -> SanitizedResult<T> {
        error::sanitize_with_error_func(
            |msg| {
                error!(self.logger(), "{}", msg);
            },
            r,
        )
    }

    pub fn detach(self) -> DetachedSession {
        DetachedSession { logger: self.inner_logger }
    }

    pub fn with_read_cache(&self) -> Session {
        Session {
            app: self.app,
            store: self.store.with_read_cache(),
            inner_logger: self.inner_logger.clone(),
        }
    }

    pub fn with_eventually_consistent_reads(&self) -> Session {
        Session {
            app: self.app,
            store: self.store.with_eventually_consistent_reads(),
            inner_logger: self.inner_logger.clone(),
        }
    }
}

// Represents a session that is momentarily detached from the App. This is useful when you need
// sessions to traverse third-party API boundaries that require 'static lifetimes.
#[derive(Clone)]
pub struct DetachedSession {
    logger: slog::Logger,
}

impl DetachedSession {
    pub fn attach(self, app: &App) -> Session {
        Session {
            app,
            store: app.store.clone(),
            inner_logger: self.logger,
        }
    }
}
