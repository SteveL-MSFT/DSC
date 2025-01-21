// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use thiserror::Error;

#[derive(Error, Debug)]
#[allow(clippy::module_name_repetitions)]
pub enum RegistryError {
    #[error("Hive not opened")]
    HiveNotOpened,

    #[error("Invalid operation: {0}.")]
    InvalidOperation(String),

    #[error("Unsupported registry value type: {0}.")]
    UnsupportedRegistryValueType(u32),

    #[error("Windows: {0}")]
    Windows(#[from] windows_result::Error),
}
