/*
 * Copyright (c) Gabriel Amihalachioaie, SimpleG 2023.
 */

use crate::models::config_supply_chain::ConfigSupplyChain;
use crate::timeouts::Timeouts;
use cp_core::auth::authorization::Authorization;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config_supply_chain: Arc<ConfigSupplyChain>,
    pub timeouts: Timeouts,
    pub authorization: Authorization,
}
