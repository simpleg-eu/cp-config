/*
 * Copyright (c) Gabriel Amihalachioaie, SimpleG 2023.
 */

use serde::Deserialize;

/// Timeouts for different functions in seconds.
#[derive(Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Timeouts {
    pub controllers_config_get_config: u64,
}
