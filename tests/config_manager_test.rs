/*
 * Copyright (c) Gabriel Amihalachioaie, SimpleG 2023.
 */

use std::path::PathBuf;
use std::sync::Arc;

use cp_core::test_base::get_test_data_path;

use cp_config::config_builder::ConfigBuilder;
use cp_config::config_manager::ConfigManager;
use cp_config::downloader::Downloader;
use cp_config::microconfig_config_builder::MicroconfigConfigBuilder;

use crate::test_base::get_git_downloader;

mod test_base;

const WORKING_DIR: &str = "./working_dir";

#[test]
pub fn setup_builds_all_environments() {
    let environments: Vec<String> = vec![
        "dummy".to_string(),
        "development".to_string(),
        "staging".to_string(),
        "production".to_string(),
    ];
    let downloader: Arc<dyn Downloader> = Arc::new(get_git_downloader(get_test_data_path(file!())));
    let builder: Arc<dyn ConfigBuilder> = Arc::new(MicroconfigConfigBuilder::default());
    let working_path: PathBuf = WORKING_DIR.into();
    let config_manager =
        ConfigManager::new(environments.clone(), downloader, builder, working_path);

    let setup_result = config_manager.setup("dummy".to_string());

    assert!(setup_result.is_ok());
    for environment in environments {
        assert!(std::fs::metadata(format!("{}/{}", WORKING_DIR, environment)).is_ok());
        assert!(std::fs::metadata(format!(
            "{}/{}/dummy/application.yaml",
            WORKING_DIR, environment
        ))
        .is_ok());
    }
}
