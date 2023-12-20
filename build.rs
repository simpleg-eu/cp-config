/*
 * Copyright (c) Gabriel Amihalachioaie, SimpleG 2023.
 */
use std::path::Path;

fn main() {
    let source_path = Path::new("config.yaml");

    let dest_path = Path::new(&std::env::var("OUT_DIR").unwrap())
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("config.yaml");

    if let Err(err) = std::fs::copy(source_path, dest_path) {
        eprintln!("error copying file: {}", err);
    } else {
        println!("file copied successfully!");
    }
}
