/*
 * Copyright (c) Gabriel Amihalachioaie, SimpleG 2023.
 */

use std::path::PathBuf;
use std::process::Command;

use cp_core::error::Error;

use crate::error_kind::{COMMAND_READ_FAILURE, CONFIG_BUILD_FAILURE, PATH_CONVERSION_ERROR};
use crate::services::config_builder::ConfigBuilder;

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
                ));
            }
        };

        let target_path_str: &str = match target_path.to_str() {
            Some(target_path_str) => target_path_str,
            None => {
                return Err(Error::new(
                    PATH_CONVERSION_ERROR.to_string(),
                    "failed to convert 'target_path' to a string.".to_string(),
                ));
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
                    ));
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

#[cfg(test)]
pub mod tests {
    use cp_core::config_reader::ConfigReader;
    use cp_core::test_base::get_unit_test_data_path;

    use crate::services::config_builder::ConfigBuilder;
    use crate::services::microconfig_config_builder::MicroconfigConfigBuilder;

    #[test]
    pub fn build_creates_expected_file() {
        let builder = MicroconfigConfigBuilder::default();
        let test_data_path = get_unit_test_data_path(file!());
        let mut build_output_path =
            std::env::current_dir().expect("failed to retrieve current path");
        build_output_path.push("__config__build__");
        let dummy_environment: &str = "dummy";
        let expected_inner_value: i64 = 1234;
        let expected_outer_value: i64 = 1;

        let build_result =
            builder.build(dummy_environment, test_data_path, build_output_path.clone());
        let mut result_file_path = build_output_path.clone();
        result_file_path.push("dummy");
        result_file_path.push("application.yaml");
        let config_reader: ConfigReader = ConfigReader::default();
        let config = config_reader.read(result_file_path).unwrap();
        std::fs::remove_dir_all(build_output_path).expect("failed to delete build path");
        let inner_value = config
            .get("example")
            .unwrap()
            .get("innerValue")
            .unwrap()
            .as_i64()
            .unwrap();
        let outer_value = config.get("outerValue").unwrap().as_i64().unwrap();

        assert!(build_result.is_ok());
        assert_eq!(expected_inner_value, inner_value);
        assert_eq!(expected_outer_value, outer_value);
    }
}
