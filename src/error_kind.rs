/*
 * Copyright (c) Gabriel Amihalachioaie, SimpleG 2023.
 */

pub const GIT_ERROR: &str = "git_error";
pub const PATH_CONVERSION_ERROR: &str = "path_conversion_error";
pub const CONFIG_BUILD_FAILURE: &str = "config_build_failure";
pub const COMMAND_READ_FAILURE: &str = "command_read_failure";
pub const FILE_NOT_FOUND: &str = "file_not_found";
pub const FAILED_TO_READ: &str = "failed_to_read";
pub const FAILED_TO_DELETE_FILE: &str = "failed_to_delete_file";
pub const CHANNEL_COMMUNICATION_FAILURE: &str = "channel_communication_failure";
pub const UNEXPECTED_RESPONSE_TYPE: &str = "unexpected_response_type";
pub const NOT_FOUND: &str = "not_found";

pub fn is_error_kind_clients_fault(error_kind: &str) -> bool {
    if error_kind == NOT_FOUND {
        return true;
    }

    return false;
}
