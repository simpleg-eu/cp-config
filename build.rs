/*
 * Copyright (c) Gabriel Amihalachioaie, SimpleG 2023.
 */
use std::path::Path;

fn main() {
    let build_mode = match option_env!("CARGO_CFG_DEBUG").map_or(false, |s| s == "true") {
        true => "debug",
        false => "release",
    };

    // Path to the file you want to copy
    let source_path = Path::new("config.yaml");

    // Destination path in the build directory
    let dest_path = Path::new(&std::env::var("OUT_DIR").unwrap())
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("config.yaml");

    // Attempt to copy the file
    if let Err(err) = std::fs::copy(&source_path, &dest_path) {
        eprintln!("error copying file: {}", err);
    } else {
        println!("file copied successfully!");
    }
}
