/*
 * Copyright (c) Gabriel Amihalachioaie, SimpleG 2023.
 */

use crate::models::config_supply_response::ConfigSupplyResponse;
use tokio::sync::oneshot::Sender;

pub enum ConfigSupplyRequest {
    Update {
        replier: Sender<ConfigSupplyResponse>,
    },
    GetConfig {
        environment: String,
        component: String,
        replier: Sender<ConfigSupplyResponse>,
    },
}
