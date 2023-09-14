// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::dscerror;

pub struct Concat {}

impl Function for Concat {
    fn invoke(&manager: FunctionManager, args: &Vec<FunctionParameter>) -> Result<FunctionParameter, dscerror::DscError> {
        if args.len() < 2 {
            return Err(dscerror::DscError::InvalidFunctionParameterCount(2, args.len()));
        }

        let mut result = String::new();
        for arg in args {
            result.push_str(value.as_str());
        }

        Ok(FunctionParameter::String(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strings() {
        let manager = FunctionManager::new();
        let result = manager.try_invoke("[concat('a', 'b')]").unwrap();
        assert_eq!(result, FunctionParameter::String("ab".to_string()));
    }

    #[test]
    fn strings_with_spaces() {
        let manager = FunctionManager::new();
        let result = manager.try_invoke("[concat('a ', ' ', ' b')]").unwrap();
        assert_eq!(result, FunctionParameter::String("a   b".to_string()));
    }

    #[test]
    fn numbers() {
        let manager = FunctionManager::new();
        let result = manager.try_invoke("[concat(1, 2)]").unwrap();
        assert_eq!(result, FunctionParameter::String("12".to_string()));
    }

    #[test]
    fn string_and_numbers() {
        let manager = FunctionManager::new();
        let result = manager.try_invoke("[concat('a', 1, 'b', 2)]").unwrap();
        assert_eq!(result, FunctionParameter::String("a1b2".to_string()));
    }

    #[test]
    fn nested() {
        let manager = FunctionManager::new();
        let result = manager.try_invoke("[concat('a', concat('b', 'c'), 'd')]").unwrap();
        assert_eq!(result, FunctionParameter::String("abcd".to_string()));
    }
}
