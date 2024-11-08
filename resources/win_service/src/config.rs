// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum ServiceType {
    #[default]
    Unknown,
    KernelDriver,
    FileSystemDriver,
    Win32OwnProcess,
    Win32ShareProcess,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum StartupType {
    #[default]
    Unknown,
    Boot,
    System,
    Automatic,
    Demand,
    Disabled,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum ErrorControl {
    #[default]
    Unknown,
    Ignore,
    Normal,
    Severe,
    Critical,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Service {
    #[schemars(extend("x-keyProperty" = true))]
    pub name: Option<String>,
    #[serde(rename = "displayName", skip_deserializing)]
    pub display_name: Option<String>,
    #[serde(rename = "imagePath", skip_deserializing)]
    pub image_path: Option<String>,
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "serviceType", skip_deserializing)]
    pub service_type: Option<ServiceType>,
    #[serde(rename = "startType")]
    pub start_type: Option<StartupType>,
    #[serde(rename = "errorControl")]
    pub error_control: Option<ErrorControl>,
    #[serde(rename = "systemRoot", skip_serializing_if = "Option::is_none")]
    pub system_root: Option<String>,
}
