use crate::Error as BoxError;
use std::{fmt, io};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    // Returned when a change conflicts with a simultaneous write, e.g. when you attempt to add a
    // revision that already exists.
    Contention,
    UserEmailAddressInUse,
    UserHandleInUse,
    TranscodingServiceExternalIdUnavailable,
    Other(BoxError),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Contention => write!(f, "contention"),
            Self::UserEmailAddressInUse => write!(f, "user email address in use"),
            Self::UserHandleInUse => write!(f, "user handle in use"),
            Self::TranscodingServiceExternalIdUnavailable => write!(f, "transcoding service external id unavailable"),
            Self::Other(err) => err.fmt(f),
        }
    }
}

impl From<BoxError> for Error {
    fn from(err: BoxError) -> Self {
        Self::Other(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::Other(Box::new(err))
    }
}

impl From<redis::RedisError> for Error {
    fn from(err: redis::RedisError) -> Self {
        Self::Other(Box::new(err))
    }
}

impl From<rmp_serde::encode::Error> for Error {
    fn from(err: rmp_serde::encode::Error) -> Self {
        Self::Other(Box::new(err))
    }
}

impl From<rmp_serde::decode::Error> for Error {
    fn from(err: rmp_serde::decode::Error) -> Self {
        Self::Other(Box::new(err))
    }
}

impl From<keyvaluestore::Error> for Error {
    fn from(err: keyvaluestore::Error) -> Self {
        match err {
            keyvaluestore::Error::AtomicWriteConflict => Self::Contention,
            keyvaluestore::Error::Other(e) => Self::Other(e),
        }
    }
}
