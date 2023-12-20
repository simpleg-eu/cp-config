/*
 * Copyright (c) Gabriel Amihalachioaie, SimpleG 2023.
 */

use std::time::Duration;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use serde::Deserialize;
use tokio::time::timeout;

use crate::app_state::AppState;

#[derive(Clone, Debug, Deserialize)]
pub struct GetConfigQueryParams {
    pub environment: String,
    pub component: String,
}

pub async fn get_config(
    Query(params): Query<GetConfigQueryParams>,
    State(state): State<AppState>,
) -> Result<(StatusCode, Vec<u8>), (StatusCode, String)> {
    let result = match timeout(
        Duration::from_secs(state.timeouts.controllers_config_get_config),
        state
            .config_supply_chain
            .get_config(&params.environment, &params.component),
    )
    .await
    {
        Ok(result) => result,
        Err(error) => return Err((StatusCode::REQUEST_TIMEOUT, format!("{}", error))),
    };

    match result {
        Ok(config) => Ok((StatusCode::OK, config)),
        Err(error) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", error))),
    }
}
