/*
 * Copyright (c) Gabriel Amihalachioaie, SimpleG 2023.
 */

use cp_config::services::config_builder::ConfigBuilder;
use cp_config::services::microconfig_config_builder::MicroconfigConfigBuilder;
use cp_core::config_reader::ConfigReader;
use cp_core::test_base::get_test_data_path;

#[test]
pub fn build_creates_expected_file() {
    let builder = MicroconfigConfigBuilder::default();
    let test_data_path = get_test_data_path(file!());
    let mut build_output_path = std::env::current_dir().expect("failed to retrieve current path");
    build_output_path.push("__config__build__");
    let dummy_environment: &str = "dummy";
    let expected_inner_value: i64 = 1234;
    let expected_outer_value: i64 = 1;

    let build_result = builder.build(dummy_environment, test_data_path, build_output_path.clone());
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
