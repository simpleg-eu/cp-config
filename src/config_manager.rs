/*
 * Copyright (c) Gabriel Amihalachioaie, SimpleG 2023.
 */

use std::path::PathBuf;
use std::sync::Arc;

use cp_core::error::Error;

use crate::cleaner::clean_working_directory;
use crate::config_builder::ConfigBuilder;
use crate::downloader::Downloader;

pub struct ConfigManager {
    environments: Vec<String>,
    downloader: Arc<dyn Downloader>,
    builder: Arc<dyn ConfigBuilder>,
    working_path: PathBuf,
}

impl ConfigManager {
    pub fn new(
        environments: Vec<String>,
        downloader: Arc<dyn Downloader>,
        builder: Arc<dyn ConfigBuilder>,
        working_path: PathBuf,
    ) -> Self {
        Self {
            environments,
            downloader,
            builder,
            working_path,
        }
    }

    pub fn setup(&self, stage: String) -> Result<(), Error> {
        let mut download_path = self.working_path.clone();
        download_path.push("download");
        self.downloader.download(download_path.clone(), stage)?;

        for environment in self.environments.as_slice() {
            let mut target_path = self.working_path.clone();
            target_path.push(environment);

            self.builder
                .build(environment, download_path.clone(), target_path)?;
        }

        Ok(())
    }
}

impl Drop for ConfigManager {
    fn drop(&mut self) {
        match clean_working_directory(&self.working_path) {
            Ok(_) => (),
            Err(error) => log::warn!("failed to clean working directory: {}", error),
        }
    }
}
