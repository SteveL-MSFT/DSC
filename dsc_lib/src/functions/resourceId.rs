// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub struct ResourceId {}

impl Function for ResourceId {
    fn invoke(&self, args: &Vec<FunctionParameter>) -> Result<FunctionParameter, dscerror::DscError> {
        if args.len() != 2 {
            return Err(dscerror::DscError::InvalidResourceId);
        }

        Ok(FunctionParameter::String(format!("{}/{}", args.get(0).unwrap(), args.get(1).unwrap())))
    }
}
