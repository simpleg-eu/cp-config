/*
 * Copyright (c) Gabriel Amihalachioaie, SimpleG 2023.
 */

use std::path::Path;

use cp_core::error::Error;

pub trait Downloader {
    ///
    /// Downloads the latest state of the configuration files which are part of the specified stage.
    ///
    /// # Arguments
    ///
    /// * `target_path` - Path where the configuration files will be downloaded.
    /// * `stage` - Flavour of the configuration files being downloaded.
    fn download(&self, target_path: &Path, stage: &str) -> Result<(), Error>;
}
