// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::dscerror;

pub struct Base64 {}

impl Function for Base64 {
    fn invoke(&manager: FunctionManager, args: &Vec<FunctionParameter>) -> Result<FunctionParameter, dscerror::DscError> {
        if args.len() != 1 {
            return Err(dscerror::DscError::InvalidFunctionParameterCount(1, args.len()));
        }

        let arg = args.get(0).unwrap();
        let value = match arg {
            FunctionParameter::String(value) => value,
            // ARM function only accepts strings
            _ => return Err(dscerror::DscError::InvalidFunctionParameter("base64".to_string(), "Expected string".to_string())),
        };

        Ok(FunctionParameter::String(base64::encode(value)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strings() {
        let manager = FunctionManager::new();
        let result = manager.try_invoke("[base64('hello world')]").unwrap();
        assert_eq!(result, FunctionParameter::String("aGVsbG8gd29ybGQ=".to_string()));
    }

    #[test]
    fn numbers() {
        let manager = FunctionManager::new();
        let result = manager.try_invoke("[base64(123)]");
        assert!(result.is_err());
    }

    #[test]
    fn nested() {
        let manager = FunctionManager::new();
        let result = manager.try_invoke("[base64(base64('hello world'))]").unwrap();
        assert_eq!(result, FunctionParameter::String("YUdWc2JHOHRZbUZ3YVdOc2JHbGhibVFnWm1Gd2FXVnVkR2x2Ym5SbGNuQnZjMlZ5ZEdsbGJuUmxjaUJqYjIwPQ==".to_string()));
    }
}
