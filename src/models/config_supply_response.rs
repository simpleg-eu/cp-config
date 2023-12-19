/*
 * Copyright (c) Gabriel Amihalachioaie, SimpleG 2023.
 */

use cp_core::error::Error;

#[derive(Debug)]
pub enum ConfigSupplyResponse {
    Update { result: Result<(), Error> },
    GetConfig { result: Result<Vec<u8>, Error> },
}
