/*
 * Copyright (c) Gabriel Amihalachioaie, SimpleG 2023.
 */

use std::path::{Path, PathBuf};

use cp_core::error::Error;
use git2::build::{CheckoutBuilder, RepoBuilder};
use git2::{
    AnnotatedCommit, AutotagOption, Cred, FetchOptions, Reference, Remote, RemoteCallbacks,
    Repository,
};

use crate::error::ConfigError;
use crate::error_kind::GIT_ERROR;
use crate::return_error;
use crate::services::downloader::Downloader;

pub struct GitDownloader {
    repository_url: String,
    username: String,
    password: String,
}

const GIT_REMOTE_NAME: &str = "origin";

impl GitDownloader {
    pub fn new(repository_url: String, username: String, password: String) -> Self {
        Self {
            repository_url,
            username,
            password,
        }
    }

    pub fn username(&self) -> &str {
        self.username.as_str()
    }

    pub fn password(&self) -> &str {
        self.password.as_str()
    }

    fn pull(&self, target_path: &Path, stage: &str) -> Result<(), Error> {
        let repository = return_error!(Repository::open(target_path));

        let mut remote = return_error!(repository.find_remote(GIT_REMOTE_NAME));

        let fetch_commit = self.fetch(&repository, &[stage], &mut remote)?;
        self.merge(&repository, stage, fetch_commit)?;

        Ok(())
    }

    fn fetch<'a>(
        &self,
        repository: &'a Repository,
        refs: &[&str],
        remote: &'a mut Remote,
    ) -> Result<AnnotatedCommit<'a>, Error> {
        let remote_callbacks = self.get_remote_callback();

        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(remote_callbacks);
        fetch_options.download_tags(AutotagOption::All);

        return_error!(remote.fetch(refs, Some(&mut fetch_options), None));

        let fetch_head = return_error!(repository.find_reference("FETCH_HEAD"));
        let commit = return_error!(repository.reference_to_annotated_commit(&fetch_head));

        Ok(commit)
    }

    fn merge<'a>(
        &self,
        repository: &'a Repository,
        remote_branch: &str,
        fetch_commit: AnnotatedCommit<'a>,
    ) -> Result<(), Error> {
        let analysis = return_error!(repository.merge_analysis(&[&fetch_commit]));

        if analysis.0.is_fast_forward() {
            let ref_name = format!("refs/heads/{}", remote_branch);

            match repository.find_reference(ref_name.as_str()) {
                Ok(mut reference) => {
                    self.fast_forward(repository, &mut reference, &fetch_commit)?;
                }
                Err(_) => {
                    self.set_reference_to_commit(
                        repository,
                        &ref_name,
                        remote_branch,
                        &fetch_commit,
                    )?;
                }
            }
        } else if analysis.0.is_normal() {
            let repository_head = return_error!(repository.head());

            let head_commit =
                return_error!(repository.reference_to_annotated_commit(&repository_head));

            return_error!(self.normal_merge(repository, &head_commit, &fetch_commit));
        }
        Ok(())
    }

    fn fast_forward(
        &self,
        repository: &Repository,
        reference: &mut Reference,
        commit: &AnnotatedCommit,
    ) -> Result<(), Error> {
        let name = match reference.name() {
            Some(name) => name.to_string(),
            None => String::from_utf8_lossy(reference.name_bytes()).to_string(),
        };

        return_error!(reference.set_target(
            commit.id(),
            &format!("fast forward: setting {} to id: {}", name, commit.id()),
        ));

        return_error!(repository.set_head(&name));

        return_error!(repository.checkout_head(Some(CheckoutBuilder::default().force())));

        Ok(())
    }

    fn set_reference_to_commit(
        &self,
        repository: &Repository,
        ref_name: &str,
        remote_branch: &str,
        fetch_commit: &AnnotatedCommit,
    ) -> Result<(), Error> {
        return_error!(repository.reference(
            ref_name,
            fetch_commit.id(),
            true,
            &format!("setting {} to {}", remote_branch, fetch_commit.id()),
        ));

        return_error!(repository.set_head(ref_name));

        return_error!(repository.checkout_head(Some(
            CheckoutBuilder::default()
                .allow_conflicts(true)
                .conflict_style_merge(true)
                .force(),
        )));

        Ok(())
    }

    fn normal_merge(
        &self,
        repository: &Repository,
        local: &AnnotatedCommit,
        remote: &AnnotatedCommit,
    ) -> Result<(), git2::Error> {
        let local_tree = repository.find_commit(local.id())?.tree()?;
        let remote_tree = repository.find_commit(remote.id())?.tree()?;
        let ancestor = repository
            .find_commit(repository.merge_base(local.id(), remote.id())?)?
            .tree()?;

        let mut index = repository.merge_trees(&ancestor, &local_tree, &remote_tree, None)?;

        if index.has_conflicts() {
            repository.checkout_index(Some(&mut index), None)?;
            return Ok(());
        }

        let result_tree = repository.find_tree(index.write_tree_to(repository)?)?;

        let message = format!("merge: {} into {}", remote.id(), local.id());
        let signature = repository.signature()?;
        let local_commit = repository.find_commit(local.id())?;
        let remote_commit = repository.find_commit(remote.id())?;

        repository.commit(
            Some("HEAD"),
            &signature,
            &signature,
            &message,
            &result_tree,
            &[&local_commit, &remote_commit],
        )?;

        repository.checkout_head(None)?;

        Ok(())
    }

    fn get_remote_callback(&self) -> RemoteCallbacks {
        let mut remote_callbacks = RemoteCallbacks::new();

        remote_callbacks.credentials(|_, _, _| {
            Cred::userpass_plaintext(self.username.as_str(), self.password.as_str())
        });

        remote_callbacks
    }

    fn clone(&self, target_path: &Path, stage: &str) -> Result<(), Error> {
        let remote_callback = self.get_remote_callback();
        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(remote_callback);

        let mut builder = RepoBuilder::new();
        builder.fetch_options(fetch_options);
        builder.branch(stage);

        return_error!(builder.clone(self.repository_url.as_str(), target_path));

        Ok(())
    }
}

impl Downloader for GitDownloader {
    ///
    /// Downloads the configuration files from the previously specified repository.
    ///
    /// # Arguments
    ///
    /// * `target_path` - Path where the configuration files will be downloaded into.
    /// * `stage` - Git branch to be used for retrieving the configuration files.
    fn download(&self, target_path: &Path, stage: &str) -> Result<(), Error> {
        let mut possible_git_repository_path = PathBuf::from(target_path);
        possible_git_repository_path.push(".git");

        // check if repository already exists
        if possible_git_repository_path.metadata().is_ok() {
            self.pull(target_path, stage)
        } else {
            self.clone(target_path, stage)
        }
    }

    fn is_new_version_available(&self, target_path: &Path, stage: &str) -> Result<bool, Error> {
        let repository = return_error!(Repository::open(target_path));

        let local_head = return_error!(repository.head());
        let local_commit_oid = return_error!(local_head.peel_to_commit()).id();

        let mut remote = return_error!(repository.find_remote("origin"));

        let mut fetch_options = FetchOptions::new();
        let mut callbacks = RemoteCallbacks::new();

        callbacks.credentials(|_, _, _| Cred::userpass_plaintext(self.username(), self.password()));

        fetch_options.remote_callbacks(callbacks);

        let empty_slice: &[&str] = &[];
        return_error!(remote.fetch(empty_slice, Some(&mut fetch_options), None));

        let remote_head = return_error!(
            repository.find_reference(format!("refs/remotes/origin/{}", stage).as_str())
        );
        let remote_commit_oid = match remote_head.target() {
            Some(oid) => oid,
            None => {
                return Err(Error::new(
                    GIT_ERROR.to_string(),
                    "failed to get oid of remote head".to_string(),
                ));
            }
        };

        Ok(local_commit_oid != remote_commit_oid)
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use chrono::Utc;
    use cp_core::error::Error;
    use cp_core::test_base::get_unit_test_data_path;
    use git2::{Repository, Signature, Time};

    use crate::error::ConfigError;
    use crate::services::downloader::Downloader;
    use crate::services::git_downloader::GitDownloader;
    use crate::test_base::get_git_downloader;

    const TEST_STAGE: &str = "dummy";

    #[test]
    pub fn download_downloads_expected_files() {
        let (result, working_directory, expected_file_exists, expected_file_exists_too) =
            download();

        std::fs::remove_dir_all(working_directory).unwrap();
        assert!(result.is_ok());
        assert!(expected_file_exists);
        assert!(expected_file_exists_too);
    }

    #[test]
    pub fn download_twice_succeeds() {
        let (_, first_working_directory, _, _) = download();
        let (result, working_directory, expected_file_exists, expected_file_exists_too) =
            download();

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
        let version_result =
            alt_downloader.is_new_version_available(&alt_download_path, TEST_STAGE);

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
        let downloader: GitDownloader = get_git_downloader(get_unit_test_data_path(file!()));
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
}
