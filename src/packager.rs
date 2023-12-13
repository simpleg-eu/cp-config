/*
 * Copyright (c) Gabriel Amihalachioaie, SimpleG 2023.
 */

use cp_core::error::Error;
use std::path::Path;

pub trait Packager {
    fn package(&self, environment: &str, component: &str, target_file: &Path) -> Result<(), Error>;
    fn extension(&self) -> String;
}
