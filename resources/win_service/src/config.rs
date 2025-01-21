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
    pub name: String,
    #[serde(rename = "displayName", skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(rename = "imagePath", skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub image_path: Option<String>,
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "serviceType", skip_deserializing, skip_serializing_if = "Option::is_none")]
    #[allow(clippy::struct_field_names)]
    pub service_type: Option<ServiceType>,
    #[serde(rename = "startType", skip_serializing_if = "Option::is_none")]
    pub start_type: Option<StartupType>,
    #[serde(rename = "errorControl", skip_serializing_if = "Option::is_none")]
    pub error_control: Option<ErrorControl>,
    #[serde(rename = "systemRoot", skip_serializing_if = "Option::is_none")]
    pub system_root: Option<String>,
}
