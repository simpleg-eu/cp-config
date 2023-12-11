/*
 * Copyright (c) Gabriel Amihalachioaie, SimpleG 2023.
 */

use cp_core::error::Error;
use std::path::PathBuf;

pub trait ConfigBuilder {
    fn build(
        &self,
        environment: &str,
        source_path: PathBuf,
        target_path: PathBuf,
    ) -> Result<(), Error>;
}
