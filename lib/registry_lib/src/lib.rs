// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use error::RegistryError;
use key::OfflineRegistryKey;
use registry::RegKey;
use rust_i18n::{i18n, t};
use std::{fmt::{Display, Formatter}, path::Path, ptr};
use tracing::debug;
use windows_result::{Error, HRESULT}; 
use windows_sys::Wdk::System::{OfflineRegistry, OfflineRegistry::ORHKEY}; 

pub mod error;
pub mod key;

i18n!("locales", fallback = "en-us");

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

pub enum Permissions {
    Read,
    Write,
    QueryValue,
    SetValue,
    CreateSubKey,
    EnumerateSubKeys,
}

pub struct Value {
    pub name: String,
    pub data: Option<ValueData>,
}

pub enum Hive {
    NotConnected,
    Offline(ORHKEY),
    Live(RegKey),
}

pub enum RegistryKey {
    Offline(OfflineRegistryKey),
    Live(RegKey),
}

pub struct Registry {
    // This is only used for offline hives.
    system_drive: String,
    hive: Hive,
    key: Option<RegistryKey>,
}

impl Registry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            system_drive: String::new(),
            hive: Hive::NotConnected,
            key: None,
        }
    }

    /// Set the offline system drive.
    /// 
    /// # Arguments
    /// 
    /// * `system_root` - The path to the target offline system root.  This can be a drive letter or a path to a mounted filesystem.
    /// 
    /// # Returns
    /// 
    /// Returns a result indicating success or failure.
    pub fn set_offline_system_root(&mut self, system_root: &str) -> Result<(), RegistryError> {
        self.close();
        if !Path::new(&system_root).exists() {
            return Err(RegistryError::InvalidOperation(t!("lib.systemRootNotFound", drive = system_root).to_string()));
        }
        self.system_drive = system_root.to_string();
        Ok(())
        // let mut path: Vec<u16> = path.encode_utf16().collect();
        // path.push(0);
        // let mut hive: ORHKEY = ptr::null_mut();
        // let result = unsafe { OfflineRegistry::OROpenHive(path.as_ptr(), &mut hive) };
        // if result != 0 {
        //     Err(RegistryError::Windows(Error::from_hresult(HRESULT::from_win32(result))))
        // } else {
        //     self.hive = Hive::Offline(hive);
        //     debug!("{}", t!("hive.hiveLoaded"));
        //     Ok(())
        // }
    }

    /// Close the hive.
    pub fn close(&mut self) {
        if let Hive::Offline(hive) = self.hive {
            unsafe { OfflineRegistry::ORCloseHive(hive) };
        }
        self.key = None;
        self.hive = Hive::NotConnected;
    }

    fn open_key(&self, path: &str, permissions: Permissions) -> Result<RegistryKey, RegistryError> {
        debug!("{}", t!("hive.openKey", path = path));
        match self.hive {
            Hive::Offline(hive) => {
                let 
            },
            Hive::Live(ref hive) => hive.open_key(path),
            Hive::NotConnected => Err(RegistryError::InvalidOperation(t!("hive.hiveNotLoaded").to_string())),
        }


        let mut key_handle: ORHKEY = ptr::null_mut();
        let mut path: Vec<u16> = path.encode_utf16().collect();
        path.push(0);
        let result = unsafe { OfflineRegistry::OROpenKey(self.hive, path.as_ptr(), &mut key_handle) };
        if result != 0 {
            Err(RegistryError::Windows(Error::from_hresult(HRESULT::from_win32(result))))
        } else {
            Ok(OfflineRegistryKey::new(path, self.hive, key_handle))
        }
    }

    /// Enumerate subkeys of a key.
    /// 
    /// # Arguments
    /// 
    /// * `path` - The path to the key to enumerate.
    /// 
    /// # Returns
    /// 
    /// Returns a vector of subkey names.
    pub fn enumerate_keys(&self, path: &str) -> Result<Vec<String>, RegistryError> {
        let mut keys = Vec::<String>::new();
        match self.hive {
            Hive::Offline(_) => {
                let key = self.open_offline_key(path)?;
                keys = key.enumerate_subkeys()?;
            },
            Hive::Live(ref hive) => {
                let key = hive.open_key(path)?;
                keys = key.enumerate_subkeys()?;
            },
            Hive::NotConnected => return Err(RegistryError::HiveNotOpened),
        }
        Ok(keys)
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

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Registry {
    fn drop(&mut self) {
        self.close();
    }
}
