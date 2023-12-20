/*
 * Copyright (c) Gabriel Amihalachioaie, SimpleG 2023.
 */

use crate::models::config_supply_chain::ConfigSupplyChain;
use crate::timeouts::Timeouts;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config_supply_chain: Arc<ConfigSupplyChain>,
    pub timeouts: Timeouts,
}
