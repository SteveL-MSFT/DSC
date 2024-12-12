// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use error::RegistryError;
use hive::OfflineRegistryHive;
use key::OfflineRegistryKey;
use registry::Hive;
use registry::RegKey;
use std::fmt::{Display, Formatter};

pub mod error;
pub mod hive;
pub mod key;

pub enum ValueData {
    None,
    Binary(Vec<u8>),
    Dword(u32),
    ExpandString(String),
    MultiString(Vec<String>),
    Qword(u64),
    String(String),
}

impl Display for ValueData {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            ValueData::None => write!(f, ""),
            ValueData::Binary(data) => write!(f, "{data:?}"),
            ValueData::Dword(data) => write!(f, "{data}"),
            ValueData::MultiString(data) => write!(f, "{data:?}"),
            ValueData::Qword(data) => write!(f, "{data}"),
            ValueData::String(data) | ValueData::ExpandString(data) => write!(f, "{data}"),
        }
    }
}

pub struct Value {
    pub name: String,
    pub data: Option<ValueData>,
}

pub struct Registry {
    offline_registry_hive: Option<OfflineRegistryHive>,
    offline_registry_key: Option<OfflineRegistryKey>,
    online_registry_hive: Option<Hive>,
    online_registry_key: Option<RegKey>,
    is_offline: bool,
}

impl Registry {
    pub fn new() -> Self {
        Registry {
            offline_registry_hive: None,
            offline_registry_key: None,
            online_registry_hive: None,
            online_registry_key: None,
            is_offline: false,
        }
    }

    pub fn is_offline(&self) -> bool {
        self.is_offline
    }

    pub fn load_offline_hive(&mut self, path: &str) -> Result<(), RegistryError> {
        self.offline_registry_hive = Some(OfflineRegistryHive::new());
        self.offline_registry_hive.as_mut().unwrap().load(path)?;
        self.is_offline = true;
        Ok(())
    }

    pub fn enumerate_keys(&self, path: &str) -> Result<Vec<String>, RegistryError> {
        if self.is_offline {
            self.offline_registry_hive.as_ref().unwrap().enumerate_keys(path)
        } else {
            self.online_registry.as_ref().unwrap().enumerate_keys(path)
        }
    }

    pub fn enumerate_values(&self, path: &str) -> Result<Vec<String>, RegistryError> {
        if self.is_offline {
            self.offline_registry.as_ref().unwrap().enumerate_values(path)
        } else {
            self.online_registry.as_ref().unwrap().enumerate_values(path)
        }
    }

    pub fn get_value(&self, path: &str, value: &str) -> Result<Vec<u8>, RegistryError> {
        if self.is_offline {
            self.offline_registry.as_ref().unwrap().get_value(path, value)
        } else {
            self.online_registry.as_ref().unwrap().get_value(path, value)
        }
    }

    pub fn set_value(&mut self, path: &str, value: &str, data: &[u8]) -> Result<(), RegistryError> {
        if self.is_offline {
            self.offline_registry.as_mut().unwrap().set_value(path, value, data)
        } else {
            self.online_registry.as_mut().unwrap().set_value(path, value, data)
        }
    }

    pub fn create_key(&mut self, path: &str) -> Result<(), RegistryError> {
        if self.is_offline {
            self.offline_registry.as_mut().unwrap().create_key(path)
        } else {
            self.online_registry.as_mut().unwrap().create_key(path)
        }
    }

    pub fn delete_key(&mut self, path: &str) -> Result<(), RegistryError> {
        if self.is_offline {
            self.offline_registry.as_mut().unwrap().delete_key(path)
        } else {
            self.online_registry.as_mut().unwrap().delete_key(path)
        }
    }
}

impl Drop for Registry {
    fn drop(&mut self) {
        if self.is_offline {
            if let Some(key) = self.offline_registry_key {
                key.close();
                self.offline_registry_key = None;
            }
            if let Some(hive) = self.offline_registry_hive {
                hive.close();
                self.offline_registry_hive = None;
            }
        } else {
            if let Some(key) = self.online_registry_key {
                key.();
                self.online_registry_key = None;
            }
            if let Some(hive) = self.online_registry_hive {
                hive.close();
                self.online_registry_hive = None;
            }
        }
    }
}