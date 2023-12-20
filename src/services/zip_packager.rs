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

#[cfg(test)]
pub mod tests {
    use std::fs::File;
    use std::io::Error;
    use std::path::Path;

    use cp_core::test_base::get_unit_test_data_path;
    use zip::ZipArchive;

    use crate::services::packager::Packager;
    use crate::services::zip_packager::ZipPackager;

    #[test]
    pub fn package_creates_zip_containing_configuration_file() {
        let working_path = get_unit_test_data_path(file!());
        let zip_packager = ZipPackager::default();
        let mut source_path = working_path.clone();
        source_path.push("dummy");
        source_path.push("dummy");
        let package_file: &str = "dummy.zip";
        let package_file_path: &Path = Path::new("dummy.zip");
        let application_file = "application.yaml";

        let _ = zip_packager.package(&source_path, package_file_path);
        let package_file_metadata = std::fs::metadata(package_file);
        let _ = unzip_file(package_file, "./");
        let config_file_metadata = std::fs::metadata(application_file);
        let _ = std::fs::remove_file(package_file);
        let _ = std::fs::remove_file(application_file);

        assert!(package_file_metadata.is_ok());
        assert!(config_file_metadata.is_ok());
    }

    fn unzip_file(zip_path: &str, extract_path: &str) -> Result<(), Error> {
        let file = File::open(zip_path)?;
        let mut archive = ZipArchive::new(file)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let output_path = format!("{}/{}", extract_path, file.name());

            if file.is_dir() {
                std::fs::create_dir_all(&output_path)?;
            } else {
                if let Some(parent_dir) = Path::new(&output_path).parent() {
                    std::fs::create_dir_all(parent_dir)?;
                }

                let mut output_file = File::create(&output_path)?;

                // Copy the content of the file to the output file
                std::io::copy(&mut file, &mut output_file)?;
            }
        }

        Ok(())
    }
}
