/*
 * Copyright (c) Gabriel Amihalachioaie, SimpleG 2023.
 */

use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use cp_core::error::Error;
use zip::{CompressionMethod, ZipWriter};

use crate::error_kind::PATH_CONVERSION_ERROR;
use crate::services::packager::Packager;

#[derive(Default)]
pub struct ZipPackager {}

impl ZipPackager {
    fn zip_directory(&self, source_path: &str, target_file: &Path) -> Result<(), std::io::Error> {
        let zip_file = File::create(target_file)?;
        let options =
            zip::write::FileOptions::default().compression_method(CompressionMethod::Stored);
        let mut zip = ZipWriter::new(zip_file);

        let mut buffer = Vec::new();

        for entry in std::fs::read_dir(source_path)? {
            let entry = entry?;
            let path = entry.path();

            let mut file = File::open(&path)?;
            file.read_to_end(&mut buffer)?;

            let result = path.strip_prefix(source_path);
            let relative_path = result.unwrap().to_string_lossy().into_owned();
            zip.start_file(relative_path, options)?;
            zip.write_all(&buffer)?;
            buffer.clear();
        }

        zip.finish()?;

        Ok(())
    }
}

impl Packager for ZipPackager {
    fn package(&self, source_path: &Path, target_file: &Path) -> Result<(), Error> {
        let source_path = match source_path.to_str() {
            Some(source_path) => source_path,
            None => {
                return Err(Error::new(
                    PATH_CONVERSION_ERROR.to_string(),
                    "failed to convert 'source_path' to string".to_string(),
                ));
            }
        };

        self.zip_directory(source_path, target_file)?;

        Ok(())
    }

    fn extension(&self) -> String {
        "zip".to_string()
    }
}
