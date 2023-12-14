/*
 * Copyright (c) Gabriel Amihalachioaie, SimpleG 2023.
 */

use cp_core::error::Error;
use std::path::Path;

pub trait Downloader {
    fn download(&self, target_path: &Path, stage: &str) -> Result<(), Error>;
}
