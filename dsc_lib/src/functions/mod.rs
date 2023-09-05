// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::dscerror;
use regex::Regex;

pub enum FunctionParameter {
    Array(Vec<FunctionParameter>),
    Bool(bool),
    Int(i32),
    String(String),
}

pub trait Function {
    fn invoke(&self, args: &Vec<FunctionParameter>) -> Result<FunctionParameter, dscerror::DscError>;
}

/// Invoke an expression with the given value.
///
/// # Arguments
///
/// * `value` - The unprocessed value to invoke as an expression.
///
/// # Returns
///
/// If `value` is a valid expression, the expression will be invoked and the result returned.
/// Otherwise, the original value will be returned.
pub fn invoke_expression(value: &str) -> Result<String, dscerror::DscError> {
    let expression_regex = Regex::new(r"^\s*\[?<function>([^\]]+)\]\s*$")?;

}
