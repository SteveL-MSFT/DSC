// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{AcceptedArgKind, Function, FunctionCategory};
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct Concat {}

impl Function for Concat {
    fn description(&self) -> String {
        t!("functions.concat.description").to_string()
    }

    fn category(&self) -> FunctionCategory {
        FunctionCategory::String
    }

    fn min_args(&self) -> usize {
        2
    }

    fn max_args(&self) -> usize {
        usize::MAX
    }

    fn accepted_arg_types(&self) -> Vec<AcceptedArgKind> {
        vec![AcceptedArgKind::String, AcceptedArgKind::Array]
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.concat.invoked"));
        let mut string_result = String::new();
        let mut array_result: Vec<String> = Vec::new();
        let mut input_type : Option<AcceptedArgKind> = None;
        for value in args {
            if value.is_string() {
                if input_type.is_none() {
                    input_type = Some(AcceptedArgKind::String);
                } else if input_type != Some(AcceptedArgKind::String) {
                    return Err(DscError::Parser(t!("functions.concat.argsMustBeStrings").to_string()));
                }

                string_result.push_str(value.as_str().unwrap_or_default());
            } else if value.is_array() {
                if input_type.is_none() {
                    input_type = Some(AcceptedArgKind::Array);
                } else if input_type != Some(AcceptedArgKind::Array) {
                    return Err(DscError::Parser(t!("functions.concat.argsMustBeArrays").to_string()));
                }

                if let Some(array) = value.as_array() {
                    for arg in array {
                        if arg.is_string() {
                            if arg.as_str().is_some() {
                                array_result.push(arg.as_str().unwrap().to_string());
                            } else {
                                array_result.push(String::new());
                            }
                        } else {
                            return Err(DscError::Parser(t!("functions.concat.onlyArraysOfStrings").to_string()));
                        }
                    }
                }
            } else {
                return Err(DscError::Parser(t!("functions.invalidArgType").to_string()));
            }
        }

        match input_type {
            Some(AcceptedArgKind::String) => {
                Ok(Value::String(string_result))
            },
            Some(AcceptedArgKind::Array) => {
                Ok(Value::Array(array_result.into_iter().map(Value::String).collect()))
            },
            _ => {
                Err(DscError::Parser(t!("functions.invalidArgType").to_string()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;

    #[test]
    fn strings() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[concat('a', 'b')]", &Context::new()).unwrap();
        assert_eq!(result, "ab");
    }

    #[test]
    fn strings_with_spaces() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[concat('a ', ' ', ' b')]", &Context::new()).unwrap();
        assert_eq!(result, "a   b");
    }

    #[test]
    fn arrays() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[concat(createArray('a','b'), createArray('c','d'))]", &Context::new()).unwrap();
        assert_eq!(result.to_string(), r#"["a","b","c","d"]"#);
    }

    #[test]
    fn string_and_numbers() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[concat('a', 1)]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn nested() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[concat('a', concat('b', 'c'), 'd')]", &Context::new()).unwrap();
        assert_eq!(result, "abcd");
    }

    #[test]
    fn invalid_one_parameter() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[concat('a')]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn string_and_array() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[concat('a', createArray('b','c'))]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn array_and_string() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[concat(createArray('a','b'), 'c')]", &Context::new());
        assert!(result.is_err());
    }
}
