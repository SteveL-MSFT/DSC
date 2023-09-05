// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub struct ResourceId {
    pub resource_type: String,
    pub resource_name: String,
}

impl Function for ResourceId {
    fn invoke(&self, args: &Vec<FunctionParameter>) -> Result<FunctionParameter, dscerror::DscError> {
        if args.len() != 2 {
            return Err(dscerror::DscError::InvalidResourceId);
        }

        // if an argument is a function, we need to call it first


        Ok(FunctionParameter::String(resource_id))
    }
}
