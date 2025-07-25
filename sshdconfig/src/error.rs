// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SshdConfigError {
    #[error("{t}: {0}", t = t!("error.command"))]
    CommandError(String),
    #[error("{t}: {0}", t = t!("error.invalidInput"))]
    InvalidInput(String),
    #[error("{t}: {0}", t = t!("error.json"))]
    Json(#[from] serde_json::Error),
    #[error("{t}: {0}", t = t!("error.language"))]
    LanguageError(#[from] tree_sitter::LanguageError),
    #[error("{t}: {0}", t = t!("error.notImplemented"))]
    NotImplemented(String),
    #[error("{t}: {0}", t = t!("error.parser"))]
    ParserError(String),
    #[error("{t}: {0}", t = t!("error.parseInt"))]
    ParseIntError(#[from] std::num::ParseIntError),
    #[cfg(windows)]
    #[error("{t}: {0}", t = t!("error.registry"))]
    RegistryError(#[from] registry_lib::error::RegistryError),
}
