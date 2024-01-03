/*
 * Copyright (c) Gabriel Amihalachioaie, SimpleG 2023.
 */

use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use cp_core::config_reader::ConfigReader;
use cp_core::secrets::get_secrets_manager;

use crate::services::git_downloader::GitDownloader;

pub fn get_git_downloader(mut config_path: PathBuf) -> GitDownloader {
    thread::sleep(Duration::from_millis(500u64));
    config_path.push("config/config.yaml");

    let config_reader: ConfigReader = ConfigReader::default();
    let config = config_reader.read(config_path).unwrap();
    let git_config = config.get("Git").unwrap();
    let repository = git_config.get("Repository").unwrap().as_str().unwrap();
    let username_secret = git_config.get("UsernameSecret").unwrap().as_str().unwrap();
    let password_secret = git_config.get("PasswordSecret").unwrap().as_str().unwrap();
    let secrets_manager = get_secrets_manager().unwrap();
    let username = secrets_manager.get_secret(username_secret).unwrap();
    let password = secrets_manager.get_secret(password_secret).unwrap();

    GitDownloader::new(repository.to_string(), username, password)
}
