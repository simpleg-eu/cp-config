/*
 * Copyright (c) Gabriel Amihalachioaie, SimpleG 2023.
 */

use std::path::PathBuf;

use cp_core::error::Error;

pub trait Downloader {
    fn download(&self, target_path: &PathBuf, stage: &str) -> Result<(), Error>;
}
