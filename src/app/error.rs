use rusoto_core;
use rusoto_credential;

use crate::{store, Error};
use std::fmt;

#[derive(Clone, Debug)]
pub enum SanitizedError {
    Unauthorized,
    NotFound(String),
    Contention,
    InternalError,
    IncorrectRevisionNumber,
    UserError(String),
}

pub fn user_error<S: Into<String>>(message: S) -> SanitizedError {
    SanitizedError::UserError(message.into())
}

impl std::error::Error for SanitizedError {}

impl fmt::Display for SanitizedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unauthorized => write!(f, "You are not authorized to perform this operation."),
            Self::NotFound(message) => write!(f, "{}", message),
            Self::Contention => write!(f, "Operation failed due to contention. Please try again."),
            Self::InternalError => write!(f, "An internal error has occurred."),
            Self::IncorrectRevisionNumber => write!(f, "This object has been modified. Please try again using the latest revision."),
            Self::UserError(message) => write!(f, "{}", message),
        }
    }
}

pub type SanitizedResult<T> = std::result::Result<T, SanitizedError>;

pub trait Sanitizable {
    fn sanitize(self) -> (SanitizedError, Option<Error>);
}

macro_rules! sanitizable_errors {
    ($t:ty) => {
        impl Sanitizable for $t {
            fn sanitize(self) -> (SanitizedError, Option<Error>) {
                (SanitizedError::InternalError, Some(Box::new(self)))
            }
        }
    };
    ($t:ty, $($r:ty),+) => {
        sanitizable_errors!($t);
        sanitizable_errors!($($r),+);
    };
}

impl Sanitizable for Error {
    fn sanitize(self) -> (SanitizedError, Option<Error>) {
        (SanitizedError::InternalError, Some(self))
    }
}

sanitizable_errors!(
    simple_error::SimpleError,
    std::io::Error,
    rmp_serde::encode::Error,
    rmp_serde::decode::Error,
    serde_json::Error,
    rusoto_credential::CredentialsError,
    rusoto_core::request::TlsError,
    tokio::task::JoinError
);

impl Sanitizable for store::Error {
    fn sanitize(self) -> (SanitizedError, Option<Error>) {
        match self {
            store::Error::UserHandleInUse => (SanitizedError::UserError("The provided handle is already in use.".to_string()), None),
            store::Error::UserEmailAddressInUse => (SanitizedError::UserError("The provided email address is already in use.".to_string()), None),
            store::Error::TranscodingServiceExternalIdUnavailable => (SanitizedError::UserError("The provided external id is invalid.".to_string()), None),
            err @ store::Error::Contention => (SanitizedError::Contention, Some(Box::new(err))),
            store::Error::Other(err) => (SanitizedError::InternalError, Some(err)),
        }
    }
}

pub fn sanitize<T, E: Sanitizable>(logger: &slog::Logger, r: std::result::Result<T, E>) -> SanitizedResult<T> {
    sanitize_with_error_func(|msg| error!(logger, "{}", msg), r)
}

pub fn sanitize_with_error_func<F: FnOnce(&str), T, E: Sanitizable>(log_error: F, r: std::result::Result<T, E>) -> SanitizedResult<T> {
    match r {
        Ok(v) => Ok(v),
        Err(e) => match e.sanitize() {
            (e @ SanitizedError::Contention, _) => Err(e),
            (e, Some(cause)) => {
                log_error(format!("{}", &cause).as_str());
                Err(e)
            }
            (e, _) => Err(e),
        },
    }
}
