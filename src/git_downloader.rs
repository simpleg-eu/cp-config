/*
 * Copyright (c) Gabriel Amihalachioaie, SimpleG 2023.
 */

use std::path::PathBuf;

use cp_core::error::Error;
use git2::build::{CheckoutBuilder, RepoBuilder};
use git2::{
    AnnotatedCommit, AutotagOption, Cred, FetchOptions, Reference, Remote, RemoteCallbacks,
    Repository,
};

use crate::downloader::Downloader;
use crate::error::ConfigError;
use crate::error_kind::GIT_ERROR;

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

    fn pull(&self, target_path: PathBuf, stage: String) -> Result<(), Error> {
        let repository = match Repository::open(target_path) {
            Ok(repository) => repository,
            Err(error) => return Err(ConfigError::from(error).into()),
        };

        let mut remote = match repository.find_remote(GIT_REMOTE_NAME) {
            Ok(remote) => remote,
            Err(error) => return Err(ConfigError::from(error).into()),
        };

        let fetch_commit = self.fetch(&repository, &[stage.as_str()], &mut remote)?;
        self.merge(&repository, &stage, fetch_commit)?;

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

        match remote.fetch(refs, Some(&mut fetch_options), None) {
            Ok(_) => (),
            Err(error) => return Err(ConfigError::from(error).into()),
        }

        let fetch_head = match repository.find_reference("FETCH_HEAD") {
            Ok(fetch_head) => fetch_head,
            Err(error) => return Err(ConfigError::from(error).into()),
        };
        let commit = match repository.reference_to_annotated_commit(&fetch_head) {
            Ok(commit) => commit,
            Err(error) => return Err(ConfigError::from(error).into()),
        };

        Ok(commit)
    }

    fn merge<'a>(
        &self,
        repository: &'a Repository,
        remote_branch: &str,
        fetch_commit: AnnotatedCommit<'a>,
    ) -> Result<(), Error> {
        let analysis = match repository.merge_analysis(&[&fetch_commit]) {
            Ok(analysis) => analysis,
            Err(error) => return Err(ConfigError::from(error).into()),
        };

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
            let repository_head = match repository.head() {
                Ok(repository_head) => repository_head,
                Err(error) => return Err(ConfigError::from(error).into()),
            };

            let head_commit = match repository.reference_to_annotated_commit(&repository_head) {
                Ok(head_commit) => head_commit,
                Err(error) => return Err(ConfigError::from(error).into()),
            };

            match self.normal_merge(repository, &head_commit, &fetch_commit) {
                Ok(_) => (),
                Err(error) => return Err(ConfigError::from(error).into()),
            }
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

        match reference.set_target(
            commit.id(),
            &format!("fast forward: setting {} to id: {}", name, commit.id()),
        ) {
            Ok(_) => (),
            Err(error) => return Err(ConfigError::from(error).into()),
        }

        match repository.set_head(&name) {
            Ok(_) => (),
            Err(error) => return Err(ConfigError::from(error).into()),
        }

        match repository.checkout_head(Some(CheckoutBuilder::default().force())) {
            Ok(_) => (),
            Err(error) => return Err(ConfigError::from(error).into()),
        }

        Ok(())
    }

    fn set_reference_to_commit(
        &self,
        repository: &Repository,
        ref_name: &str,
        remote_branch: &str,
        fetch_commit: &AnnotatedCommit,
    ) -> Result<(), Error> {
        match repository.reference(
            ref_name,
            fetch_commit.id(),
            true,
            &format!("setting {} to {}", remote_branch, fetch_commit.id()),
        ) {
            Ok(_) => (),
            Err(error) => return Err(ConfigError::from(error).into()),
        }

        match repository.set_head(ref_name) {
            Ok(_) => (),
            Err(error) => return Err(ConfigError::from(error).into()),
        }

        match repository.checkout_head(Some(
            CheckoutBuilder::default()
                .allow_conflicts(true)
                .conflict_style_merge(true)
                .force(),
        )) {
            Ok(_) => (),
            Err(error) => return Err(ConfigError::from(error).into()),
        }

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

    fn clone(&self, target_path: PathBuf, stage: String) -> Result<(), Error> {
        let remote_callback = self.get_remote_callback();
        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(remote_callback);

        let mut builder = RepoBuilder::new();
        builder.fetch_options(fetch_options);
        builder.branch(stage.as_str());

        match builder.clone(self.repository_url.as_str(), target_path.as_path()) {
            Ok(_) => (),
            Err(error) => {
                return Err(Error::new(
                    GIT_ERROR.to_string(),
                    format!("failed to clone repository: {}", error.message()),
                ));
            }
        }

        Ok(())
    }
}

impl Downloader for GitDownloader {
    fn download(&self, target_path: PathBuf, stage: String) -> Result<(), Error> {
        let mut possible_git_repository_path = target_path.clone();
        possible_git_repository_path.push(".git");

        // check if repository already exists
        if possible_git_repository_path.metadata().is_ok() {
            self.pull(target_path, stage)
        } else {
            self.clone(target_path, stage)
        }
    }
}
