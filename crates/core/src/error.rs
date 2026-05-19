use std::env::VarError;
use std::num::ParseIntError;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, ConfigError>;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("required environment variable {name} is not set")]
    MissingEnv { name: &'static str },

    #[error("environment variable {name} must not be empty")]
    EmptyEnv { name: &'static str },

    #[error("environment variable {name} contains non-unicode data")]
    InvalidUnicode { name: &'static str },

    #[error("environment variable {name} has invalid port value {value:?}")]
    InvalidPort {
        name: &'static str,
        value: String,
        #[source]
        source: ParseIntError,
    },
}

impl ConfigError {
    pub(crate) fn from_var_error(name: &'static str, error: VarError) -> Self {
        match error {
            VarError::NotPresent => Self::MissingEnv { name },
            VarError::NotUnicode(_) => Self::InvalidUnicode { name },
        }
    }
}
