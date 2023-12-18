/*
 * Copyright (c) Gabriel Amihalachioaie, SimpleG 2023.
 */

use std::path::PathBuf;

use cp_core::error::Error;
#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait ConfigBuilder {
    ///
    /// Builds the configuration files for the specified environment, using the source files located at the specified
    /// source path, locating the build results into the target path.
    ///
    /// # Arguments
    ///
    /// * `environment` - Environment of configuration files to be used, i.e. development, staging, production.
    /// * `source_path` - Path containing the root of the configuration files.
    /// * `target_path` - Path which will contain the resulting built configuration files.
    fn build(
        &self,
        environment: &str,
        source_path: PathBuf,
        target_path: PathBuf,
    ) -> Result<(), Error>;
}
