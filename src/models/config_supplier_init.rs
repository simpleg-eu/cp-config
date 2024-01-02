/*
 * Copyright (c) Gabriel Amihalachioaie, SimpleG 2023.
 */

use std::sync::Arc;

use crate::services::config_builder::ConfigBuilder;
use crate::services::downloader::Downloader;
use crate::services::packager::Packager;

pub struct ConfigSupplierInit {
    pub environments: Vec<String>,
    pub downloader: Arc<dyn Downloader + Send + Sync>,
    pub builder: Arc<dyn ConfigBuilder + Send + Sync>,
    pub packager: Arc<dyn Packager + Send + Sync>,
}
