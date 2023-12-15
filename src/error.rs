/*
 * Copyright (c) Gabriel Amihalachioaie, SimpleG 2023.
 */

use crate::error_kind::GIT_ERROR;
use cp_core::error::Error;

pub struct ConfigError(Error);

impl From<git2::Error> for ConfigError {
    fn from(value: git2::Error) -> Self {
        Self(Error::new(
            GIT_ERROR.to_string(),
            value.message().to_string(),
        ))
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(value: std::io::Error) -> Self {
        Self(Error::new(value.kind().to_string(), value.to_string()))
    }
}

impl From<ConfigError> for Error {
    fn from(value: ConfigError) -> Self {
        value.0
    }
}
