/*
 * Copyright (c) Gabriel Amihalachioaie, SimpleG 2023.
 */

mod test_base;

use cp_core::config_reader::ConfigReader;
use cp_core::error::Error;
use cp_core::secrets::bitwarden_secrets_manager::BitwardenSecretsManager;
use cp_core::secrets::secrets_manager::SecretsManager;
use cp_core::test_base::get_test_data_path;

use crate::test_base::get_git_downloader;
use cp_config::downloader::Downloader;
use cp_config::git_downloader::GitDownloader;

fn download() -> (Result<(), Error>, String, bool, bool) {
    let working_directory: &str = "./working-dir";
    std::fs::create_dir(working_directory);
    let downloader: GitDownloader = get_git_downloader(get_test_data_path(file!()));
    let result = downloader.download(
        format!("{}/download", working_directory).into(),
        "dummy".to_string(),
    );
    let expected_file_exists: bool = std::fs::metadata(format!(
        "{}/download/dummy/this_file_must_exist.yaml",
        working_directory
    ))
    .is_ok();
    let expected_file_exists_too: bool = std::fs::metadata(format!(
        "{}/download/dummy/this_file_must_exist.yaml",
        working_directory
    ))
    .is_ok();

    (
        result,
        working_directory.to_string(),
        expected_file_exists,
        expected_file_exists_too,
    )
}

#[test]
pub fn download_downloads_expected_files() {
    let (result, working_directory, expected_file_exists, expected_file_exists_too) = download();

    std::fs::remove_dir_all(working_directory).unwrap();
    assert!(result.is_ok());
    assert!(expected_file_exists);
    assert!(expected_file_exists_too);
}

#[test]
pub fn download_twice_succeeds() {
    download();
    let (result, working_directory, expected_file_exists, expected_file_exists_too) = download();

    std::fs::remove_dir_all(working_directory).unwrap();
    assert!(result.is_ok());
    assert!(expected_file_exists);
    assert!(expected_file_exists_too);
}
