/*
 * Copyright (c) Gabriel Amihalachioaie, SimpleG 2023.
 */
use std::path::Path;

fn main() {
    let config_source_path = Path::new("config.yaml");
    let log_source_path = Path::new("log4rs.yaml");

    let out_dir = &std::env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(out_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    let config_dest_path = dest_path.join("config.yaml");
    let log_dest_path = dest_path.join("log4rs.yaml");

    match std::fs::copy(config_source_path, config_dest_path) {
        Ok(_) => (),
        Err(error) => eprintln!("error copying configuration file: {}", error),
    }

    match std::fs::copy(log_source_path, log_dest_path) {
        Ok(_) => (),
        Err(error) => eprintln!("error copying log configuration file: {}", error),
    }
}
