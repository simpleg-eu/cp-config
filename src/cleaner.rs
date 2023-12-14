/*
 * Copyright (c) Gabriel Amihalachioaie, SimpleG 2023.
 */

use std::path::Path;

use cp_core::error::Error;

pub fn clean_working_directory(working_directory: &Path) -> Result<(), Error> {
    match std::fs::remove_dir_all(working_directory) {
        Ok(_) => Ok(()),
        Err(error) => Err(error.into()),
    }
}

#[cfg(test)]
pub mod tests {
    use crate::cleaner::clean_working_directory;
    use std::path::Path;

    #[test]
    pub fn clean_working_directory_deletes_specified_directory() {
        let working_directory = format!("./{}", uuid::Uuid::new_v4());
        std::fs::create_dir(&working_directory).unwrap();
        std::fs::File::create(format!("{}/example.txt", working_directory)).unwrap();
        clean_working_directory(Path::new(&working_directory)).unwrap();

        assert!(std::fs::metadata(&working_directory).is_err());
    }
}
