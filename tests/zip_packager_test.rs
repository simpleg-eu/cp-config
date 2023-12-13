/*
 * Copyright (c) Gabriel Amihalachioaie, SimpleG 2023.
 */

use cp_config::packager::Packager;
use cp_config::zip_packager::ZipPackager;
use cp_core::test_base::get_test_data_path;
use std::fs::File;
use std::io::Error;
use std::path::Path;
use zip::ZipArchive;

#[test]
pub fn package_creates_zip_containing_configuration_file() {
    let working_path = get_test_data_path(file!());
    let zip_packager = ZipPackager::new(working_path.clone());
    let environment: &str = "dummy";
    let component: &str = "dummy";
    let package_file: &str = "dummy.zip";
    let package_file_path: &Path = Path::new("dummy.zip");
    let application_file = "application.yaml";

    zip_packager.package(environment, component, package_file_path);
    let package_file_metadata = std::fs::metadata(package_file);
    unzip_file(package_file, "./");
    let config_file_metadata = std::fs::metadata(application_file);
    std::fs::remove_file(package_file);
    std::fs::remove_file(application_file);

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
            if let Some(parent_dir) = std::path::Path::new(&output_path).parent() {
                std::fs::create_dir_all(parent_dir)?;
            }

            let mut output_file = File::create(&output_path)?;

            // Copy the content of the file to the output file
            std::io::copy(&mut file, &mut output_file)?;
        }
    }

    Ok(())
}
