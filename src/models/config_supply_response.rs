/*
 * Copyright (c) Gabriel Amihalachioaie, SimpleG 2023.
 */

use cp_core::error::Error;

pub enum ConfigSupplyResponse {
    UpdateResult { result: Result<(), Error> },
}
