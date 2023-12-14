/*
 * Copyright (c) Gabriel Amihalachioaie, SimpleG 2023.
 */

use std::path::PathBuf;

use cp_core::error::Error;
use cp_core::test_base::get_test_data_path;

use cp_config::downloader::Downloader;
use cp_config::git_downloader::GitDownloader;

use crate::test_base::get_git_downloader;

mod test_base;

fn download() -> (Result<(), Error>, String, bool, bool) {
    let working_directory = format!("./{}", uuid::Uuid::new_v4());
    std::fs::create_dir_all(&working_directory);
    let downloader: GitDownloader = get_git_downloader(get_test_data_path(file!()));
    let download_path: PathBuf = format!("{}/download", &working_directory).into();
    let result = downloader.download(&download_path, "dummy");
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
        working_directory,
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
    let (_, first_working_directory, _, _) = download();
    let (result, working_directory, expected_file_exists, expected_file_exists_too) = download();

    std::fs::remove_dir_all(first_working_directory).unwrap();
    std::fs::remove_dir_all(working_directory).unwrap();
    assert!(result.is_ok());
    assert!(expected_file_exists);
    assert!(expected_file_exists_too);
}
