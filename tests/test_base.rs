/*
 * Copyright (c) Gabriel Amihalachioaie, SimpleG 2023.
 */

use cp_config::services::git_downloader::GitDownloader;
use cp_core::config_reader::ConfigReader;
use cp_core::secrets::bitwarden_secrets_manager::BitwardenSecretsManager;
use cp_core::secrets::secrets_manager::SecretsManager;
use std::path::PathBuf;

pub fn get_git_downloader(mut config_path: PathBuf) -> GitDownloader {
    config_path.push("config.yaml");

    let config_reader: ConfigReader = ConfigReader::default();
    let config = config_reader.read(config_path).unwrap();
    let git_config = config.get("Git").unwrap();
    let repository = git_config.get("Repository").unwrap().as_str().unwrap();
    let username_secret = git_config.get("UsernameSecret").unwrap().as_str().unwrap();
    let password_secret = git_config.get("PasswordSecret").unwrap().as_str().unwrap();
    let secrets_manager =
        BitwardenSecretsManager::new(std::env::var("SECRETS_MANAGER_ACCESS_TOKEN").unwrap());
    let username = secrets_manager.get_secret(username_secret).unwrap();
    let password = secrets_manager.get_secret(password_secret).unwrap();

    GitDownloader::new(repository.to_string(), username, password)
}
