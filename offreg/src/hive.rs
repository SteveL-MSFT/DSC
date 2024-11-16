// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use core::ptr;
use crate::error::OfflineRegistryError;
use crate::key::OfflineRegistryKey;
use tracing::debug;
use windows_result::{Error, HRESULT};
use windows_sys::Wdk::System::{OfflineRegistry, OfflineRegistry::ORHKEY};

#[allow(clippy::module_name_repetitions)]
pub struct OfflineRegistryHive {
    hive: ORHKEY,
}

impl OfflineRegistryHive {
    #[must_use]
    pub fn new() -> Self {
        Self {
            hive: ptr::null_mut(),
        }
    }

    /// Load a hive from a file.
    /// 
    /// # Arguments
    /// 
    /// * `hive_path` - The path to the hive file.
    /// 
    /// # Returns
    /// 
    /// Returns `Ok(())` if the hive was loaded successfully.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the hive could not be loaded.
    pub fn load(&mut self, hive_path: &str) -> Result<(), OfflineRegistryError> {
        debug!("Loading hive: {hive_path}");
        let mut path: Vec<u16> = hive_path.encode_utf16().collect();
        path.push(0);
        let result = unsafe { OfflineRegistry::OROpenHive(path.as_ptr(), &mut self.hive) };
        if result != 0 {
            Err(OfflineRegistryError::Windows(Error::from_hresult(HRESULT::from_win32(result))))
        } else {
            debug!("Hive loaded");
            Ok(())
        }
    }

    /// Close the hive.
    pub fn close(&mut self) {
        unsafe { OfflineRegistry::ORCloseHive(self.hive) };
        self.hive = ptr::null_mut();
    }

    /// Open a key.
    /// 
    /// # Arguments
    /// 
    /// * `key_path` - The path to the key to open.
    /// 
    /// # Returns
    /// 
    /// Returns a new `OfflineRegistryKey` if the key was opened successfully.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the key could not be opened.
    pub fn open_key(&self, key_path: &str) -> Result<OfflineRegistryKey, OfflineRegistryError> {
        if self.hive.is_null() {
            return Err(OfflineRegistryError::InvalidOperation("Hive not loaded".to_string()));
        }

        debug!("Opening key: {key_path}");
        let mut key_handle = ptr::null_mut();
        let mut path: Vec<u16> = key_path.encode_utf16().collect();
        path.push(0);
        let result = unsafe { OfflineRegistry::OROpenKey(self.hive, path.as_ptr(), &mut key_handle) };
        if result != 0 {
            Err(OfflineRegistryError::Windows(Error::from_hresult(HRESULT::from_win32(result))))
        } else {
            Ok(OfflineRegistryKey::new(key_path, key_handle))
        }
    }
}

impl Drop for OfflineRegistryHive {
    fn drop(&mut self) {
        self.close();
    }
}

impl Default for OfflineRegistryHive {
    fn default() -> Self {
        Self::new()
    }
}
