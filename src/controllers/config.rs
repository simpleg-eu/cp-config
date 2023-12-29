/*
 * Copyright (c) Gabriel Amihalachioaie, SimpleG 2023.
 */

use std::time::Duration;

use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use cp_core::auth::error_kind::INVALID_TOKEN;
use cp_core::authorize;
use serde::Deserialize;
use tokio::time::timeout;

use crate::app_state::AppState;
use crate::error_kind::is_error_kind_clients_fault;

#[derive(Clone, Debug, Deserialize)]
pub struct GetConfigQueryParams {
    pub environment: String,
    pub component: String,
}

pub async fn get_config(
    headers: HeaderMap,
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

    authorize!(state.authorization, headers);

    match result {
        Ok(config) => Ok((StatusCode::OK, config)),
        Err(error) => {
            if is_error_kind_clients_fault(error.error_kind()) {
                return Err((StatusCode::BAD_REQUEST, format!("{}", error)));
            }

            Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", error)))
        }
    }
}
