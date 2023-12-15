/*
 * Copyright (c) Gabriel Amihalachioaie, SimpleG 2023.
 */

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;

use cp_core::error::Error;
use cp_core::ok_or_return_error;

use crate::error_kind::{FAILED_TO_DELETE_FILE, FAILED_TO_READ, FILE_NOT_FOUND};
use crate::services::cleaner::clean_working_directory;
use crate::services::config_builder::ConfigBuilder;
use crate::services::downloader::Downloader;
use crate::services::packager::Packager;

pub struct ConfigManager {
    environments: Vec<String>,
    downloader: Arc<dyn Downloader>,
    builder: Arc<dyn ConfigBuilder>,
    packager: Arc<dyn Packager>,
    working_path: PathBuf,
}

impl ConfigManager {
    pub fn new(
        environments: Vec<String>,
        downloader: Arc<dyn Downloader>,
        builder: Arc<dyn ConfigBuilder>,
        packager: Arc<dyn Packager>,
        working_path: PathBuf,
    ) -> Self {
        Self {
            environments,
            downloader,
            builder,
            packager,
            working_path,
        }
    }

    pub fn setup(&self, stage: &str) -> Result<(), Error> {
        let download_path: PathBuf = self.get_download_path();
        self.downloader.download(&download_path, stage)?;

        for environment in self.environments.as_slice() {
            let mut target_path = self.working_path.clone();
            target_path.push(environment);

            self.builder
                .build(environment, download_path.clone(), target_path)?;
        }

        Ok(())
    }

    pub fn get_config(&self, environment: &str, component: &str) -> Result<Vec<u8>, Error> {
        let mut package_file_path = self.working_path.clone();
        package_file_path.push(environment);
        package_file_path.push(component);
        package_file_path.push(format!(
            "{}.{}",
            uuid::Uuid::new_v4(),
            self.packager.extension()
        ));

        let mut source_path = self.working_path.clone();
        source_path.push(environment);
        source_path.push(component);

        self.packager
            .package(&source_path, package_file_path.as_path())?;

        let mut package_file = ok_or_return_error!(
            File::open(&package_file_path),
            FILE_NOT_FOUND.to_string(),
            "failed to open package file: "
        );

        let mut buffer: Vec<u8> = Vec::new();

        ok_or_return_error!(
            package_file.read_to_end(&mut buffer),
            FAILED_TO_READ.to_string(),
            "failed to read package file: "
        );

        ok_or_return_error!(
            std::fs::remove_file(package_file_path),
            FAILED_TO_DELETE_FILE.to_string(),
            "failed to read package file: "
        );

        Ok(buffer)
    }

    pub fn is_new_version_available(&self, stage: &str) -> Result<bool, Error> {
        let download_path = self.get_download_path();
        self.downloader
            .is_new_version_available(&download_path, stage)
    }

    fn get_download_path(&self) -> PathBuf {
        let mut download_path = self.working_path.clone();
        download_path.push("download");

        download_path
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
