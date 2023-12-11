/*
 * Copyright (c) Gabriel Amihalachioaie, SimpleG 2023.
 */

use crate::config_builder::ConfigBuilder;
use crate::error_kind::{COMMAND_READ_FAILURE, CONFIG_BUILD_FAILURE, PATH_CONVERSION_ERROR};
use cp_core::error::Error;
use std::path::PathBuf;
use std::process::Command;

#[derive(Default)]
pub struct MicroconfigConfigBuilder {}

impl ConfigBuilder for MicroconfigConfigBuilder {
    fn build(
        &self,
        environment: &str,
        source_path: PathBuf,
        target_path: PathBuf,
    ) -> Result<(), Error> {
        let source_path_str: &str = match source_path.to_str() {
            Some(source_path_str) => source_path_str,
            None => {
                return Err(Error::new(
                    PATH_CONVERSION_ERROR.to_string(),
                    "failed to convert 'source_path' to a string.".to_string(),
                ))
            }
        };

        let target_path_str: &str = match target_path.to_str() {
            Some(target_path_str) => target_path_str,
            None => {
                return Err(Error::new(
                    PATH_CONVERSION_ERROR.to_string(),
                    "failed to convert 'target_path' to a string.".to_string(),
                ))
            }
        };

        let output = match Command::new("microconfig")
            .args([
                "-r",
                source_path_str,
                "-e",
                environment,
                "-d",
                target_path_str,
            ])
            .output()
        {
            Ok(output) => output,
            Err(error) => return Err(error.into()),
        };

        if !output.status.success() {
            let error_message = match String::from_utf8(output.stderr) {
                Ok(error_message) => error_message,
                Err(error) => {
                    return Err(Error::new(
                        COMMAND_READ_FAILURE.to_string(),
                        format!(
                            "failed to read error message from output's stderr: {}",
                            error
                        ),
                    ))
                }
            };

            return Err(Error::new(
                CONFIG_BUILD_FAILURE.to_string(),
                format!("failed to build configuration: {}", error_message),
            ));
        }

        Ok(())
    }
}
