/*
 * Copyright (c) Gabriel Amihalachioaie, SimpleG 2024.
 */

use std::sync::Arc;

use axum::routing::get;
use axum::Router;
use cp_core::auth::authorization::Authorization;
use cp_core::auth::jwt_token_validator::{try_get_jwks, JwtTokenValidator};
use cp_core::config_reader::ConfigReader;
use cp_core::secrets::get_secrets_manager;
use serde_yaml::Value;

use crate::app_state::AppState;
use crate::models::config_supplier_init::ConfigSupplierInit;
use crate::models::config_supply_chain::ConfigSupplyChain;
use crate::services::config_builder::ConfigBuilder;
use crate::services::downloader::Downloader;
use crate::services::git_downloader::GitDownloader;
use crate::services::microconfig_config_builder::MicroconfigConfigBuilder;
use crate::services::packager::Packager;
use crate::services::zip_packager::ZipPackager;
use crate::timeouts::Timeouts;

pub mod app_state;
pub mod controllers;
pub mod error;
mod error_kind;
pub mod models;
pub mod services;
pub mod test_base;
pub mod timeouts;

#[tokio::main]
pub async fn main() {
    log4rs::init_file("log4rs.yaml", Default::default()).expect("failed to initialize logger");
    let config = get_config();

    let supply_chain = get_config_supply_chain(&config);
    let timeouts = serde_yaml::from_value::<Timeouts>(
        config
            .get("Timeouts")
            .expect("failed to get 'Timeouts' from the configuration file")
            .clone(),
    )
    .expect("failed to deserialize 'Timeouts'");

    let authorization = get_authorization(&config).await;

    let app_state = AppState {
        config_supply_chain: Arc::new(supply_chain),
        timeouts,
        authorization,
    };

    let app = Router::new()
        .route("/config", get(controllers::config::get_config))
        .with_state(app_state);
    let address = get_address(&config);
    let listener = tokio::net::TcpListener::bind(&address)
        .await
        .expect("failed to listen to port");
    axum::serve::serve(listener, app)
        .await
        .expect("failed to serve web");

    log::info!("serving cp-config at {}", address);
}

fn get_config() -> Value {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("cp-config [configuration file path]");
        panic!("invalid call, received less arguments than expected");
    }

    let config_file_path = args
        .get(1)
        .expect("failed to get configuration file path from arguments");

    let config_reader = ConfigReader::default();

    config_reader
        .read(config_file_path.into())
        .expect("failed to read the configuration file")
}

fn get_config_supply_chain(config: &Value) -> ConfigSupplyChain {
    let environments: Vec<String> = config
        .get("Environments")
        .expect("failed to get 'Environments' from the configuration file")
        .as_sequence()
        .expect("failed to read 'Environments' as a sequence")
        .iter()
        .map(|v| {
            v.as_str()
                .expect("failed to convert environment value to string")
                .to_string()
        })
        .collect();

    let secrets_manager = get_secrets_manager().expect("failed to get the secrets manager");

    let git_repository_url = config
        .get("Git")
        .expect("failed to get 'Git' from the configuration file")
        .get("Repository")
        .expect("failed to get 'Repository' from the configuration file")
        .as_str()
        .expect("failed to read 'Repository' as a string")
        .to_string();
    let username = secrets_manager
        .get_secret(
            config
                .get("Git")
                .expect("failed to get 'Git' from the configuration file")
                .get("UsernameSecret")
                .expect("failed to get 'UsernameSecret' from the configuration file")
                .as_str()
                .expect("failed to read 'UsernameSecret' as string"),
        )
        .expect("failed to get git username");
    let password = secrets_manager
        .get_secret(
            config
                .get("Git")
                .expect("failed to get 'Git' from the configuration file")
                .get("PasswordSecret")
                .expect("failed to get 'PasswordSecret' from the configuration file")
                .as_str()
                .expect("failed to read 'PasswordSecret' as string"),
        )
        .expect("failed to get git password");
    let downloader: Arc<dyn Downloader + Send + Sync> =
        Arc::new(GitDownloader::new(git_repository_url, username, password));
    let builder: Arc<dyn ConfigBuilder + Send + Sync> =
        Arc::new(MicroconfigConfigBuilder::default());
    let packager: Arc<dyn Packager + Send + Sync> = Arc::new(ZipPackager::default());
    let static_stages: Vec<String> = config
        .get("StaticStages")
        .expect("failed to get 'StaticStages' from the configuration file")
        .as_sequence()
        .expect("failed to read 'StaticStages' as sequence")
        .iter()
        .map(|v| {
            v.as_str()
                .expect("failed to read 'StaticStages' value as string")
                .to_string()
        })
        .collect();

    let supplier_init = ConfigSupplierInit {
        environments,
        downloader,
        builder,
        packager,
    };

    let config_suppliers_count = config
        .get("ConfigSuppliersCount")
        .expect("failed to get 'ConfigSuppliersCount' from the configuration file")
        .as_u64()
        .expect("failed to read 'ConfigSuppliersCount' as u64");

    ConfigSupplyChain::try_new(
        config_suppliers_count as usize,
        static_stages,
        supplier_init,
    )
    .expect("failed to get config supply chain")
}

async fn get_authorization(config: &Value) -> Authorization {
    let authorization = config
        .get("Authorization")
        .expect("failed to get 'Authorization' from the configuration file");

    let issuers: Vec<String> = authorization
        .get("Issuers")
        .expect("failed to get 'Issuers' from 'Authorization'")
        .as_sequence()
        .expect("failed to get 'Issuers' as sequence")
        .iter()
        .map(|v| {
            v.as_str()
                .expect("failed to get 'Issuers' value as string")
                .to_string()
        })
        .collect();

    let audience: Vec<String> = authorization
        .get("Audience")
        .expect("failed to get 'Audience' from 'Authorization'")
        .as_sequence()
        .expect("failed to get 'Audience' as sequence")
        .iter()
        .map(|v| {
            v.as_str()
                .expect("failed to get 'Audience' value as string")
                .to_string()
        })
        .collect();

    let jwks_uri = authorization
        .get("JwksUri")
        .expect("failed to get 'JwksUri' from 'Authorization'")
        .as_str()
        .expect("failed to get 'JwksUri' as string")
        .to_string();

    let jwk_set = try_get_jwks(jwks_uri.as_str())
        .await
        .expect("expected 'JwkSet'");

    let jwt_token_validator = JwtTokenValidator::new(jwk_set, issuers, audience);

    Authorization::new(Arc::new(jwt_token_validator))
}

fn get_address(config: &Value) -> String {
    let tcp_listener = config
        .get("TcpListener")
        .expect("failed to get 'TcpListener' from the configuration file");
    let ip = tcp_listener
        .get("Address")
        .expect("failed to get 'Address' from the configuration file")
        .as_str()
        .expect("failed to read 'Address' as a string");
    let port = tcp_listener
        .get("Port")
        .expect("failed to get 'Port' from the configuration file")
        .as_str()
        .expect("failed to read 'Port' as a string");

    format!("{}:{}", ip, port)
}
