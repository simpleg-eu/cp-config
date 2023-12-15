/*
 * Copyright (c) Gabriel Amihalachioaie, SimpleG 2023.
 */

use std::path::{Path, PathBuf};

use chrono::Utc;
use cp_core::error::Error;
use cp_core::test_base::get_test_data_path;
use git2::{Repository, Signature, Time};

use cp_config::error::ConfigError;
use cp_config::services::downloader::Downloader;
use cp_config::services::git_downloader::GitDownloader;

use crate::test_base::get_git_downloader;

mod test_base;

const TEST_STAGE: &str = "dummy";

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

#[test]
pub fn is_update_available_returns_true_when_there_is_an_update_available() {
    let (working_directory, downloader, download_path) = prepare_downloader();
    let (alt_working_directory, alt_downloader, alt_download_path) = prepare_downloader();

    let result = downloader.download(&download_path, TEST_STAGE);
    let alt_result = alt_downloader.download(&alt_download_path, TEST_STAGE);
    let write_result = write_changes(&download_path, &downloader);
    let version_result = alt_downloader.is_new_version_available(&alt_download_path, TEST_STAGE);

    let _ = std::fs::remove_dir_all(working_directory);
    let _ = std::fs::remove_dir_all(alt_working_directory);
    assert!(result.is_ok());
    assert!(alt_result.is_ok());
    assert!(write_result.is_ok());
    assert!(version_result.is_ok());
    assert!(version_result.unwrap());
}

fn download() -> (Result<(), Error>, String, bool, bool) {
    let (working_directory, downloader, download_path) = prepare_downloader();
    let result = downloader.download(&download_path, TEST_STAGE);
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

fn prepare_downloader() -> (String, GitDownloader, PathBuf) {
    let working_directory = format!("./{}", uuid::Uuid::new_v4());
    let _ = std::fs::create_dir_all(&working_directory);
    let downloader: GitDownloader = get_git_downloader(get_test_data_path(file!()));
    let download_path: PathBuf = format!("{}/download", &working_directory).into();

    (working_directory, downloader, download_path)
}

fn write_changes(path: &PathBuf, downloader: &GitDownloader) -> Result<(), ConfigError> {
    let repository = Repository::open(path)?;

    let mut example_file_path = path.clone();
    example_file_path.push(format!("{}.txt", uuid::Uuid::new_v4()));

    let example_file_content = uuid::Uuid::new_v4().to_string();
    std::fs::write(&example_file_path, example_file_content)?;

    let mut index = repository.index()?;

    index.add_path(Path::new(example_file_path.file_name().unwrap()))?;

    index.write()?;

    let tree_id = index.write_tree()?;
    let tree = repository.find_tree(tree_id)?;

    let head = repository.head()?;
    let head_commit = repository.find_commit(head.target().unwrap())?;

    let time = Time::new(Utc::now().timestamp(), 0);
    let signature = Signature::new("SimpleG", "gabriel@simpleg.eu", &time)?;

    repository.commit(
        Some("HEAD"),
        &signature,
        &signature,
        "+ random file",
        &tree,
        &[&head_commit],
    )?;

    let mut remote = repository.find_remote("origin")?;
    let mut callbacks = git2::RemoteCallbacks::new();
    callbacks.credentials(|_, _, _| {
        git2::Cred::userpass_plaintext(downloader.username(), downloader.password())
    });

    let mut push_options = git2::PushOptions::new();
    push_options.remote_callbacks(callbacks);

    remote.push(
        &["refs/heads/dummy:refs/heads/dummy"],
        Some(&mut push_options),
    )?;

    Ok(())
}
