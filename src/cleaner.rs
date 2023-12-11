/*
 * Copyright (c) Gabriel Amihalachioaie, SimpleG 2023.
 */

use std::path::PathBuf;

use cp_core::error::Error;

pub fn clean_working_directory(working_directory: PathBuf) -> Result<(), Error> {
    match std::fs::remove_dir_all(working_directory.as_path()) {
        Ok(_) => Ok(()),
        Err(error) => Err(error.into()),
    }
}

#[cfg(test)]
pub mod tests {
    use crate::cleaner::clean_working_directory;

    #[test]
    pub fn clean_working_directory_deletes_specified_directory() {
        std::fs::create_dir("./working_dir").unwrap();
        std::fs::File::create("./working_dir/example.txt").unwrap();

        clean_working_directory("./working_dir".into()).unwrap();

        assert!(std::fs::metadata("./working_dir").is_err());
    }
}
