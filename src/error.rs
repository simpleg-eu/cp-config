/*
 * Copyright (c) Gabriel Amihalachioaie, SimpleG 2023.
 */

use crate::error_kind::{CHANNEL_COMMUNICATION_FAILURE, GIT_ERROR};
use crate::models::config_supply_request::ConfigSupplyRequest;
use async_channel::SendError;
use cp_core::error::Error;
use tokio::sync::oneshot::error::RecvError;

#[macro_export]
macro_rules! return_error {
    ($expression:expr) => {
        match $expression {
            Ok(value) => value,
            Err(error) => return Err(ConfigError::from(error).into()),
        }
    };
}

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

impl From<RecvError> for ConfigError {
    fn from(value: RecvError) -> Self {
        Self(Error::new(
            CHANNEL_COMMUNICATION_FAILURE.to_string(),
            format!("failed to receive value: {}", value.to_string()),
        ))
    }
}

impl From<SendError<ConfigSupplyRequest>> for ConfigError {
    fn from(value: SendError<ConfigSupplyRequest>) -> Self {
        Self(Error::new(
            CHANNEL_COMMUNICATION_FAILURE.to_string(),
            format!("failed to send request: {}", value.to_string()),
        ))
    }
}

impl From<ConfigError> for Error {
    fn from(value: ConfigError) -> Self {
        value.0
    }
}
