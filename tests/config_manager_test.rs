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
use cp_config::packager::Packager;
use cp_config::zip_packager::ZipPackager;

use crate::test_base::get_git_downloader;

mod test_base;

#[test]
pub fn setup_builds_all_environments() {
    let working_dir = format!("./{}", uuid::Uuid::new_v4());
    let config_manager = get_config_manager(working_dir.clone());

    let setup_result = config_manager.setup("dummy");

    assert!(setup_result.is_ok());
    for environment in get_environments() {
        assert!(std::fs::metadata(format!("{}/{}", working_dir, environment)).is_ok());
        assert!(std::fs::metadata(format!(
            "{}/{}/dummy/application.yaml",
            working_dir, environment
        ))
        .is_ok());
    }
}

#[test]
pub fn get_config_returns_bytes_of_zip_file() {
    let working_dir = format!("./{}", uuid::Uuid::new_v4());
    let config_manager = get_config_manager(working_dir);
    config_manager
        .setup("dummy")
        .expect("failed to setup 'dummy' stage");

    let result = config_manager.get_config("dummy", "dummy");

    match result {
        Ok(data) => assert!(!data.is_empty()),
        Err(error) => {
            panic!("{}", error);
        }
    }
}

fn get_environments() -> Vec<String> {
    vec![
        "dummy".to_string(),
        "development".to_string(),
        "staging".to_string(),
        "production".to_string(),
    ]
}

fn get_config_manager(working_dir: String) -> ConfigManager {
    let downloader: Arc<dyn Downloader> = Arc::new(get_git_downloader(get_test_data_path(file!())));
    let builder: Arc<dyn ConfigBuilder> = Arc::new(MicroconfigConfigBuilder::default());
    let working_path: PathBuf = working_dir.into();
    let packager: Arc<dyn Packager> = Arc::new(ZipPackager::new(working_path.clone()));

    ConfigManager::new(
        get_environments(),
        downloader,
        builder,
        packager,
        working_path,
    )
}
