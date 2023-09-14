// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::dscerror;
use dscerror::DscError;
use regex::Regex;

pub enum FunctionParameter {
    Array(Vec<FunctionParameter>),
    Bool(bool),
    Int(i32),
    String(String),
}

pub trait Function {
    fn invoke(&manager: FunctionManager, args: &Vec<FunctionParameter>) -> Result<FunctionParameter, dscerror::DscError>;
}

pub struct FunctionManager {
    functions: HashMap<String, Box<dyn Function>>,
    function_regex: Regex,
    args_regex: Regex,
}

impl FunctionManager {
    pub fn new() -> FunctionManager {
        let mut functions = HashMap::new();
        functions.insert("base64".to_string(), Box::new(Base64::new()));
        functions.insert("concat".to_string(), Box::new(Concat::new()));
        functions.insert("resourceId".to_string(), Box::new(ResourceId::new()));
        let function_names = functions.keys().join("|");
        FunctionManager {
            functions,
            // functions look like this: "[function_name([arg1], [arg2], ...)]" where string args are in single quotes
            function_regex: Regex::new(format!("^\\[(?<name>({function_names}))\\((?<args>.*)\\)\\]$")).unwrap(),
            // args are either a string in single quotes or a non-comma separated value
            args_regex: Regex::new(r"(?<arg>'[^']*'|[^,]+)").unwrap(),
        }
    }

    pub fn try_invoke(&self, input: &str) -> Result<FunctionParameter, DscError> {
        let Some(captures) = self.function_regex.captures(input) else {
            // if it's not a function, we just return it
            return Ok(FunctionParameter::String(input.to_string()));
        };

        let name = captures.name("name").unwrap().as_str();
        let args = captures.name("args").unwrap().as_str();
        let mut function_args = Vec::new();
        for arg in self.args_regex.captures_iter(args) {
            let arg = arg.name("arg").unwrap().as_str();
            // arg may be a function itself, so we try to invoke it first
            function_args.push(self.try_invoke(arg));
        }

        if args.is_some() && function_args.len() == 0 {
            return Err(DscError::InvalidFunctionParameter(name.to_string(), args.unwrap().to_string()));
        }

        self.invoke(name, &function_args)
    }

    pub fn invoke(&self, name: &str, args: &Vec<FunctionParameter>) -> Result<FunctionParameter, DscError> {
        let Some(function) = self.functions.get(name) else {
            return Err(DscError::InvalidFunctionName(name.to_string()));
        };

        function.invoke(&new_args)
    }
}
