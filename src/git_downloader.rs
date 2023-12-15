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

use crate::downloader::Downloader;
use crate::error::ConfigError;
use crate::error_kind::GIT_ERROR;

macro_rules! return_error {
    ($expression:expr) => {
        match $expression {
            Ok(value) => value,
            Err(error) => return Err(ConfigError::from(error).into()),
        }
    };
}

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
