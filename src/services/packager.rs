/*
 * Copyright (c) Gabriel Amihalachioaie, SimpleG 2023.
 */

use cp_core::error::Error;
use std::path::Path;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait Packager {
    ///
    /// Packages the configuration files for the specified environment and component into the target package file.
    ///
    /// # Arguments
    ///
    /// * `environment` - Environment of configuration files to be used, i.e. development, staging, production.
    /// * `component` - Configuration component, microservice, whose configuration files will be packaged into the
    /// target package file.
    /// * `target_file` - Package file.
    fn package(&self, source_path: &Path, target_file: &Path) -> Result<(), Error>;

    ///
    /// Retrieves the extension of the resulting package file.
    ///
    fn extension(&self) -> String;
}
