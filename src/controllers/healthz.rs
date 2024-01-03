/*
 * Copyright (c) Gabriel Amihalachioaie, SimpleG 2024.
 */

use axum::extract::State;
use axum::http::StatusCode;

use crate::app_state::AppState;

pub async fn get_readiness(
    State(state): State<AppState>,
) -> Result<StatusCode, (StatusCode, String)> {
    let result = state
        .config_supply_chain
        .get_config("dummy", "development", "dummy")
        .await;

    match result {
        Ok(_) => Ok(StatusCode::OK),
        Err(error) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", error))),
    }
}

pub async fn get_liveness() -> StatusCode {
    StatusCode::OK
}
