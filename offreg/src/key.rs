// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use core::ptr;
use crate::error::OfflineRegistryError;
use std::ffi::c_void;
use std::fmt::{Display, Formatter};
use tracing::debug;
use windows_result::{Error, HRESULT};
use windows_sys::Wdk::System::{OfflineRegistry, OfflineRegistry::ORHKEY};

pub enum OfflineRegistryValueData {
    None,
    Binary(Vec<u8>),
    Dword(u32),
    ExpandString(String),
    MultiString(Vec<String>),
    Qword(u64),
    String(String),
}

impl Display for OfflineRegistryValueData {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            OfflineRegistryValueData::None => write!(f, ""),
            OfflineRegistryValueData::Binary(data) => write!(f, "{data:?}"),
            OfflineRegistryValueData::Dword(data) => write!(f, "{data}"),
            OfflineRegistryValueData::MultiString(data) => write!(f, "{data:?}"),
            OfflineRegistryValueData::Qword(data) => write!(f, "{data}"),
            OfflineRegistryValueData::String(data) | OfflineRegistryValueData::ExpandString(data) => write!(f, "{data}"),
        }
    }
}

pub struct OfflineRegistryValue {
    pub name: String,
    pub data: Option<OfflineRegistryValueData>,
}

#[allow(clippy::module_name_repetitions)]
pub struct OfflineRegistryKey {
    pub name: String,
    handle: ORHKEY,
}

impl Display for OfflineRegistryKey {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl OfflineRegistryKey {
    pub fn new(name: &str, handle: ORHKEY) -> Self {
        Self {
            name: name.to_string(),
            handle,
        }
    }

    /// Close the key.
    pub fn close(&mut self) {
        unsafe { OfflineRegistry::ORCloseKey(self.handle) };
        self.handle = ptr::null_mut();
    }

    /// Open a subkey.
    /// 
    /// # Arguments
    /// 
    /// * `subkey_name` - The name of the subkey to open.
    /// 
    /// # Returns
    /// 
    /// Returns a new `OfflineRegistryKey` if the subkey was opened successfully.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the subkey could not be opened.
    fn open_subkey(&self, subkey_name: &str) -> Result<OfflineRegistryKey, OfflineRegistryError> {
        debug!("Opening subkey: {subkey_name}");
        let mut subkey_handle: ORHKEY = ptr::null_mut();
        let mut path: Vec<u16> = subkey_name.encode_utf16().collect();
        path.push(0);
        let result = unsafe { OfflineRegistry::OROpenKey(self.handle, path.as_ptr(), &mut subkey_handle) };
        if result != 0 {
            Err(OfflineRegistryError::Windows(Error::from_hresult(HRESULT::from_win32(result))))
        } else {
            Ok(OfflineRegistryKey::new(subkey_name, subkey_handle))
        }
    }

    /// Enumerate subkeys.
    /// 
    /// # Returns
    /// 
    /// Returns a vector of `OfflineRegistryKey` objects.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the subkeys could not be enumerated.
    pub fn enumerate_subkeys(&self) -> Result<Vec<OfflineRegistryKey>, OfflineRegistryError> {
        debug!("Enumerating subkeys for: {}", self.name);
        let mut subkeys = Vec::<OfflineRegistryKey>::new();
        let mut index = 0;
        loop {
            let mut subkey_name = [0u16; 256];
            let mut subkey_name_length = u32::try_from(subkey_name.len()).unwrap_or(256);
            let result = unsafe {
                OfflineRegistry::OREnumKey(
                    self.handle,
                    index,
                    subkey_name.as_mut_ptr(),
                    &mut subkey_name_length,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                )
            };
            if result != 0 {
                debug!("EnumKey failed with error code: {}", result);
                break;
            }
            let subkey_name = String::from_utf16_lossy(&subkey_name[..subkey_name_length as usize]);
            if let Ok(subkey) = self.open_subkey(&subkey_name) { subkeys.push(subkey) } else {
                debug!("OpenSubkey failed for: {}", subkey_name);
                continue
            }
            index += 1;
        }
        Ok(subkeys)
    }

    /// Enumerate values.
    /// 
    /// # Returns
    /// 
    /// Returns a vector of `OfflineRegistryValue` objects.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the values could not be enumerated.
    pub fn enumerate_values(&self) -> Result<Vec<OfflineRegistryValue>, OfflineRegistryError> {
        let mut values = Vec::<OfflineRegistryValue>::new();
        let mut index = 0;
        loop {
            let mut value_name = [0u16; 256];
            let mut value_name_length = u32::try_from(value_name.len()).unwrap_or(256);
            let mut value_type = 0;
            let mut value_data = [0u8; 1024];
            let mut value_data_length = u32::try_from(value_data.len()).unwrap_or(1024);
            let result = unsafe {
                OfflineRegistry::OREnumValue(
                    self.handle,
                    index,
                    value_name.as_mut_ptr(),
                    &mut value_name_length,
                    &mut value_type,
                    value_data.as_mut_ptr(),
                    &mut value_data_length,
                )
            };
            if result != 0 {
                break;
            }
            let value_name = String::from_utf16_lossy(&value_name[..value_name_length as usize]);
            let value_data = convert_to_value_data(value_type, &value_data[..value_data_length as usize]);
            values.push(OfflineRegistryValue {
                name: value_name,
                data: Some(value_data),
            });
            index += 1;
        }
        Ok(values)
    }

    /// Get a value.
    /// 
    /// # Arguments
    /// 
    /// * `value_name` - The name of the value to get.
    /// 
    /// # Returns
    /// 
    /// Returns an `OfflineRegistryValue` object.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the value could not be retrieved.
    pub fn get_value(&self, value_name: &str) -> Result<OfflineRegistryValue, OfflineRegistryError> {
        let mut value_type = 0;
        let mut value_data = [0u8; 1024];
        let mut value_data_length = u32::try_from(value_data.len()).unwrap_or(1024);
        let mut name: Vec<u16> = value_name.encode_utf16().collect();
        name.push(0);
        let result = unsafe {
            OfflineRegistry::ORGetValue(
                self.handle,
                ptr::null(),
                name.as_ptr(),
                &mut value_type,
                value_data.as_mut_ptr().cast::<c_void>(),
                &mut value_data_length,
            )
        };
        if result != 0 {
            return Err(OfflineRegistryError::Windows(Error::from_hresult(HRESULT::from_win32(result))));
        }
        let value_data = convert_to_value_data(value_type, &value_data[..value_data_length as usize]);
        Ok(OfflineRegistryValue {
            name: value_name.to_string(),
            data: Some(value_data),
        })
    }
}

fn convert_to_value_data(value_type: u32, value_data: &[u8]) -> OfflineRegistryValueData {
    match value_type {
        0 => OfflineRegistryValueData::None,
        1 => OfflineRegistryValueData::String(String::from_utf8_lossy(value_data).to_string()),
        2 => OfflineRegistryValueData::ExpandString(String::from_utf8_lossy(value_data).to_string()),
        4 => OfflineRegistryValueData::Dword(u32::from_le_bytes([value_data[0], value_data[1], value_data[2], value_data[3]])),
        7 => {
            let mut multi_string = Vec::<String>::new();
            let mut start = 0;
            for i in 0..value_data.len() {
                if value_data[i] == 0 {
                    multi_string.push(String::from_utf8_lossy(&value_data[start..i]).to_string());
                    start = i + 1;
                }
            }
            OfflineRegistryValueData::MultiString(multi_string)
        }
        11 => OfflineRegistryValueData::Qword(u64::from_le_bytes([value_data[0], value_data[1], value_data[2], value_data[3], value_data[4], value_data[5], value_data[6], value_data[7]])),
        _ => OfflineRegistryValueData::Binary(value_data.to_vec()),
    }
}

impl Drop for OfflineRegistryKey {
    fn drop(&mut self) {
        self.close();
    }
}
