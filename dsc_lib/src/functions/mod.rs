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
        }
    }

    pub fn parse_args(input: &str) -> Result<Vec<FunctionParameter>, DscError> {
        let mut args = Vec::new();
        // scan the input string
        // strings are in single quotes
        // numbers are just numbers
        // text not in single quotes is a potential function
        // arguments are separated by commas
        let mut in_string = false;
        let mut in_number = false;
        let mut in_function = false;
        let mut in_arg = false;
        let mut arg_start = 0;
        let mut arg_end = 0;
        let mut arg = String::new();
        for (i, c) in input.chars().enumerate() {
            if in_string {
                if c == '\'' {
                    in_string = false;
                    args.push(FunctionParameter::String(arg));
                    arg = String::new();
                } else {
                    arg.push(c);
                }
            } else if in_number {
                if c.is_numeric() {
                    arg.push(c);
                } else {
                    in_number = false;
                    args.push(FunctionParameter::Int(arg.parse::<i32>().unwrap()));
                    arg = String::new();
                }
            } else if in_function {
                if c == '(' {
                    in_function = false;
                    in_arg = true;
                    arg_start = i;
                } else {
                    arg.push(c);
                }
            } else if in_arg {
                if c == '\'' {
                    in_string = true;
                } else if c.is_numeric() {
                    in_number = true;
                    arg.push(c);
                } else if c == ',' {
                    in_arg = false;
                    arg_end = i;
                    args.push(FunctionParameter::String(input[arg_start..arg_end].to_string()));
                } else if c == ')' {
                    in_arg = false;
                    arg_end = i;
                    args.push(FunctionParameter::String(input[arg_start..arg_end].to_string()));
                    break;
                }
            } else {
                if c == '\'' {
                    in_string = true;
                } else if c.is_numeric() {
                    in_number = true;
                    arg.push(c);
                } else if c == '[' {
                    in_function = true;
                }
            }
        }
    }

    pub fn try_invoke(&self, input: &str) -> Result<FunctionParameter, DscError> {
        // if input isn't an expression, just return it
        if !input.starts_with('[') || !input.ends_with(']') {
            return Ok(FunctionParameter::String(input.to_string()));
        }



    }

    pub fn invoke(&self, name: &str, args: &Vec<FunctionParameter>) -> Result<FunctionParameter, DscError> {
        let Some(function) = self.functions.get(name) else {
            return Err(DscError::InvalidFunctionName(name.to_string()));
        };

        function.invoke(&new_args)
    }
}
