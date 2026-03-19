// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::collections::HashMap;

use rust_i18n::t;
use windows::core::{BSTR, HSTRING, Interface};
use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, VARIANT_BOOL};
use windows::Win32::NetworkManagement::WindowsFirewall::{
    INetFwPolicy2, INetFwRule, NetFwPolicy2, NetFwRule, NET_FW_ACTION_ALLOW, NET_FW_ACTION_BLOCK,
    NET_FW_RULE_DIR_IN, NET_FW_RULE_DIR_OUT,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, IDispatch, CLSCTX_INPROC_SERVER,
    COINIT_MULTITHREADED,
};
use windows::Win32::System::Ole::IEnumVARIANT;
use windows::Win32::System::Variant::{VARIANT, VT_DISPATCH};
use windows::Win32::UI::Shell::SHLoadIndirectString;

use crate::types::{Action, Direction, FirewallError, FirewallRule, Profile, Protocol};

impl From<windows::core::Error> for FirewallError {
    fn from(e: windows::core::Error) -> Self {
        Self {
            message: e.to_string(),
        }
    }
}

/// RAII guard that calls `CoUninitialize` when dropped.
struct ComGuard;

impl Drop for ComGuard {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}

/// Initialize COM for the current thread.
fn init_com() -> Result<ComGuard, FirewallError> {
    unsafe {
        let hr = CoInitializeEx(None, COINIT_MULTITHREADED);
        if hr.is_err() {
            return Err(FirewallError::from(
                t!("get.comInitFailed", error = hr.to_string()).to_string(),
            ));
        }
    }
    Ok(ComGuard)
}

/// Create an `INetFwPolicy2` COM object for accessing the Windows Firewall.
fn get_policy() -> Result<INetFwPolicy2, FirewallError> {
    unsafe {
        CoCreateInstance(&NetFwPolicy2, None, CLSCTX_INPROC_SERVER).map_err(|e| {
            FirewallError::from(
                t!("get.policyCreateFailed", error = e.to_string()).to_string(),
            )
        })
    }
}

/// Convert a BSTR to `Option<String>`, returning `None` for empty strings.
fn bstr_to_option(bstr: &BSTR) -> Option<String> {
    if bstr.is_empty() {
        None
    } else {
        Some(bstr.to_string())
    }
}

/// Convert an optional BSTR value to `Option<String>`, treating `*` as `None`.
fn bstr_to_option_any(bstr: &BSTR) -> Option<String> {
    let s = bstr.to_string();
    if s.is_empty() || s == "*" {
        None
    } else {
        Some(s)
    }
}

/// Resolve a MUI indirect string (e.g. `@FirewallAPI.dll,-28544`) to its
/// localized value using `SHLoadIndirectString`. Returns the original string
/// unchanged if it is not an indirect reference or if resolution fails.
fn resolve_indirect_string(s: &str) -> String {
    if !s.starts_with('@') {
        return s.to_string();
    }
    let source = HSTRING::from(s);
    let mut buf = [0u16; 1024];
    if unsafe { SHLoadIndirectString(&source, &mut buf, None) }.is_ok() {
        String::from_utf16_lossy(
            &buf[..buf.iter().position(|&c| c == 0).unwrap_or(buf.len())],
        )
    } else {
        s.to_string()
    }
}

/// Read all properties from a COM `INetFwRule` into a `FirewallRule`.
///
/// # Safety
///
/// The `rule` must be a valid COM object reference.
unsafe fn read_rule_state(rule: &INetFwRule) -> FirewallRule {
    let raw_name = unsafe { rule.Name() }.unwrap_or_default();
    let raw_description = bstr_to_option(&unsafe { rule.Description() }.unwrap_or_default());
    let name = resolve_indirect_string(&raw_name.to_string());
    let description = raw_description.as_deref().map(resolve_indirect_string);
    let canonical_name = if raw_name.to_string() != name {
        Some(raw_name.to_string())
    } else {
        None
    };
    let enabled = unsafe { rule.Enabled() }.unwrap_or(VARIANT_BOOL(0));
    let direction = unsafe { rule.Direction() }.unwrap_or(NET_FW_RULE_DIR_IN);
    let action = unsafe { rule.Action() }.unwrap_or(NET_FW_ACTION_BLOCK);
    let protocol_num = unsafe { rule.Protocol() }.unwrap_or(256); // ANY
    let local_ports = bstr_to_option_any(&unsafe { rule.LocalPorts() }.unwrap_or_default());
    let remote_ports = bstr_to_option_any(&unsafe { rule.RemotePorts() }.unwrap_or_default());
    let local_addresses =
        bstr_to_option_any(&unsafe { rule.LocalAddresses() }.unwrap_or_default());
    let remote_addresses =
        bstr_to_option_any(&unsafe { rule.RemoteAddresses() }.unwrap_or_default());
    let profiles_mask = unsafe { rule.Profiles() }.unwrap_or(0x7FFF_FFFF);
    let program = bstr_to_option_any(&unsafe { rule.ApplicationName() }.unwrap_or_default());

    FirewallRule {
        name: Some(name),
        canonical_name,
        exist: Some(true),
        description,
        enabled: Some(enabled.as_bool()),
        direction: match direction {
            NET_FW_RULE_DIR_OUT => Some(Direction::Outbound),
            _ => Some(Direction::Inbound),
        },
        action: match action {
            NET_FW_ACTION_ALLOW => Some(Action::Allow),
            _ => Some(Action::Block),
        },
        protocol: Protocol::from_protocol_number(protocol_num),
        local_ports,
        remote_ports,
        local_addresses,
        remote_addresses,
        profiles: Some(Profile::from_bitmask(profiles_mask)),
        program,
    }
}

/// Look up a firewall rule by name and return its current state.
pub fn get_rule(input: &FirewallRule) -> Result<FirewallRule, FirewallError> {
    let name = input
        .name
        .as_deref()
        .ok_or_else(|| FirewallError::from(t!("get.nameRequired").to_string()))?;

    let _com = init_com()?;
    let policy = get_policy()?;
    let rules = unsafe { policy.Rules() }.map_err(|e| {
        FirewallError::from(t!("get.getRulesFailed", error = e.to_string()).to_string())
    })?;

    let bstr_name = BSTR::from(name);
    match unsafe { rules.Item(&bstr_name) } {
        Ok(rule) => Ok(unsafe { read_rule_state(&rule) }),
        Err(e) if e.code() == ERROR_FILE_NOT_FOUND.to_hresult() => Ok(FirewallRule {
            name: Some(name.to_string()),
            exist: Some(false),
            ..Default::default()
        }),
        Err(e) => Err(FirewallError::from(
            t!("get.itemFailed", error = e.to_string()).to_string(),
        )),
    }
}

/// Helper to set a single property on a COM rule, generating an error with the property name.
macro_rules! set_property {
    ($rule:expr, $method:ident, $value:expr, $prop_name:literal) => {
        unsafe {
            $rule.$method($value).map_err(|e| {
                FirewallError::from(
                    t!(
                        "set.setPropertyFailed",
                        property = $prop_name,
                        error = e.to_string()
                    )
                    .to_string(),
                )
            })?;
        }
    };
}

/// Apply desired properties to an `INetFwRule` COM object.
///
/// # Safety
///
/// The `rule` must be a valid COM object reference.
unsafe fn apply_properties(
    rule: &INetFwRule,
    input: &FirewallRule,
) -> Result<(), FirewallError> {
    if let Some(ref desc) = input.description {
        set_property!(rule, SetDescription, &BSTR::from(desc.as_str()), "description");
    }
    if let Some(enabled) = input.enabled {
        set_property!(rule, SetEnabled, VARIANT_BOOL::from(enabled), "enabled");
    }
    if let Some(ref dir) = input.direction {
        let dir_val = match dir {
            Direction::Inbound => NET_FW_RULE_DIR_IN,
            Direction::Outbound => NET_FW_RULE_DIR_OUT,
        };
        set_property!(rule, SetDirection, dir_val, "direction");
    }
    if let Some(ref action) = input.action {
        let action_val = match action {
            Action::Allow => NET_FW_ACTION_ALLOW,
            Action::Block => NET_FW_ACTION_BLOCK,
        };
        set_property!(rule, SetAction, action_val, "action");
    }
    // Protocol must be set before ports
    if let Some(ref proto) = input.protocol {
        set_property!(rule, SetProtocol, proto.to_protocol_number(), "protocol");
    }
    if let Some(ref ports) = input.local_ports {
        set_property!(rule, SetLocalPorts, &BSTR::from(ports.as_str()), "localPorts");
    }
    if let Some(ref ports) = input.remote_ports {
        set_property!(rule, SetRemotePorts, &BSTR::from(ports.as_str()), "remotePorts");
    }
    if let Some(ref addrs) = input.local_addresses {
        set_property!(rule, SetLocalAddresses, &BSTR::from(addrs.as_str()), "localAddresses");
    }
    if let Some(ref addrs) = input.remote_addresses {
        set_property!(rule, SetRemoteAddresses, &BSTR::from(addrs.as_str()), "remoteAddresses");
    }
    if let Some(ref profiles) = input.profiles {
        let mask = Profile::to_bitmask(profiles);
        set_property!(rule, SetProfiles, mask, "profiles");
    }
    if let Some(ref prog) = input.program {
        set_property!(rule, SetApplicationName, &BSTR::from(prog.as_str()), "program");
    }
    Ok(())
}

/// Set defaults for required properties when creating a new rule.
///
/// # Safety
///
/// The `rule` must be a valid COM object reference.
unsafe fn set_new_rule_defaults(
    rule: &INetFwRule,
    input: &FirewallRule,
) -> Result<(), FirewallError> {
    if input.direction.is_none() {
        set_property!(rule, SetDirection, NET_FW_RULE_DIR_IN, "direction");
    }
    if input.action.is_none() {
        set_property!(rule, SetAction, NET_FW_ACTION_ALLOW, "action");
    }
    if input.enabled.is_none() {
        set_property!(rule, SetEnabled, VARIANT_BOOL::from(true), "enabled");
    }
    Ok(())
}

/// Create, modify, or delete a firewall rule based on the desired state.
pub fn set_rule(input: &FirewallRule) -> Result<FirewallRule, FirewallError> {
    let name = input
        .name
        .as_deref()
        .ok_or_else(|| FirewallError::from(t!("set.nameRequired").to_string()))?;

    let _com = init_com()?;
    let policy = get_policy()?;
    let rules = unsafe { policy.Rules() }.map_err(|e| {
        FirewallError::from(t!("set.getRulesFailed", error = e.to_string()).to_string())
    })?;

    let desired_exist = input.exist.unwrap_or(true);
    let bstr_name = BSTR::from(name);
    let current_rule = unsafe { rules.Item(&bstr_name) };

    if !desired_exist {
        if current_rule.is_ok() {
            unsafe {
                rules.Remove(&bstr_name).map_err(|e| {
                    FirewallError::from(
                        t!("set.removeFailed", error = e.to_string()).to_string(),
                    )
                })?;
            }
        }
        return Ok(FirewallRule {
            name: Some(name.to_string()),
            exist: Some(false),
            ..Default::default()
        });
    }

    if let Ok(rule) = current_rule {
        unsafe { apply_properties(&rule, input) }?;
        Ok(unsafe { read_rule_state(&rule) })
    } else {
        create_new_rule(&rules, &bstr_name, input)
    }
}

/// Create a new firewall rule with the given properties.
fn create_new_rule(
    rules: &windows::Win32::NetworkManagement::WindowsFirewall::INetFwRules,
    bstr_name: &BSTR,
    input: &FirewallRule,
) -> Result<FirewallRule, FirewallError> {
    let new_rule: INetFwRule =
        unsafe { CoCreateInstance(&NetFwRule, None, CLSCTX_INPROC_SERVER) }.map_err(|e| {
            FirewallError::from(
                t!("set.createRuleFailed", error = e.to_string()).to_string(),
            )
        })?;

    set_property!(new_rule, SetName, bstr_name, "name");
    unsafe { set_new_rule_defaults(&new_rule, input) }?;
    unsafe { apply_properties(&new_rule, input) }?;

    unsafe {
        rules.Add(&new_rule).map_err(|e| {
            FirewallError::from(
                t!("set.addRuleFailed", error = e.to_string()).to_string(),
            )
        })?;
    }

    Ok(unsafe { read_rule_state(&new_rule) })
}

/// Extract an `INetFwRule` from a `VARIANT` returned by COM enumeration.
///
/// # Safety
///
/// The VARIANT must contain a `VT_DISPATCH` value pointing to a valid COM object.
unsafe fn extract_rule_from_variant(variant: &VARIANT) -> Result<INetFwRule, FirewallError> {
    let inner = unsafe { &variant.Anonymous.Anonymous };
    if inner.vt == VT_DISPATCH {
        let dispatch_opt: &Option<IDispatch> = unsafe { &inner.Anonymous.pdispVal };
        if let Some(dispatch) = dispatch_opt {
            return dispatch.cast::<INetFwRule>().map_err(|e| {
                FirewallError::from(
                    t!("export.castRuleFailed", error = e.to_string()).to_string(),
                )
            });
        }
    }
    Err(FirewallError::from(
        t!("export.unexpectedVariant").to_string(),
    ))
}

/// Check whether `rule` matches all non-`None` fields in `filter`.
fn matches_filter(rule: &FirewallRule, filter: &FirewallRule) -> bool {
    if let Some(ref pattern) = filter.name
        && !matches_wildcard(rule.name.as_deref().unwrap_or(""), pattern)
    {
        return false;
    }

    if let Some(ref expected_dir) = filter.direction
        && rule.direction.as_ref() != Some(expected_dir)
    {
        return false;
    }

    if let Some(ref expected_action) = filter.action
        && rule.action.as_ref() != Some(expected_action)
    {
        return false;
    }

    if let Some(expected_enabled) = filter.enabled
        && rule.enabled.unwrap_or(false) != expected_enabled
    {
        return false;
    }

    if let Some(ref expected_proto) = filter.protocol
        && rule.protocol.as_ref() != Some(expected_proto)
    {
        return false;
    }

    if let Some(ref expected_profiles) = filter.profiles {
        if let Some(ref actual_profiles) = rule.profiles {
            for ep in expected_profiles {
                if !actual_profiles.contains(ep) {
                    return false;
                }
            }
        } else {
            return false;
        }
    }

    true
}

/// Match a string against a pattern supporting `*` wildcards.
fn matches_wildcard(text: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    let text_lower = text.to_lowercase();
    let pattern_lower = pattern.to_lowercase();

    let parts: Vec<&str> = pattern_lower.split('*').collect();

    if parts.len() == 1 {
        return text_lower == pattern_lower;
    }

    let starts_with_wildcard = pattern_lower.starts_with('*');
    let ends_with_wildcard = pattern_lower.ends_with('*');

    let mut pos = 0;

    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }

        if i == 0 && !starts_with_wildcard {
            if !text_lower.starts_with(part) {
                return false;
            }
            pos = part.len();
        } else if let Some(found) = text_lower[pos..].find(part) {
            pos += found + part.len();
        } else {
            return false;
        }
    }

    if !ends_with_wildcard
        && let Some(last) = parts.last()
        && !last.is_empty()
        && !text_lower.ends_with(last)
    {
        return false;
    }

    true
}

/// Export (enumerate) all firewall rules, optionally filtering by the provided criteria.
pub fn export_rules(
    filter: Option<&FirewallRule>,
) -> Result<Vec<FirewallRule>, FirewallError> {
    let _com = init_com()?;
    let policy = get_policy()?;
    let rules = unsafe { policy.Rules() }.map_err(|e| {
        FirewallError::from(t!("export.getRulesFailed", error = e.to_string()).to_string())
    })?;

    let unknown = unsafe { rules._NewEnum() }.map_err(|e| {
        FirewallError::from(t!("export.enumFailed", error = e.to_string()).to_string())
    })?;
    let enum_var: IEnumVARIANT = unknown.cast().map_err(|e| {
        FirewallError::from(t!("export.enumFailed", error = e.to_string()).to_string())
    })?;

    let mut results = Vec::new();
    let mut names: HashMap<String, i32> = HashMap::new();

    loop {
        let mut variant = VARIANT::default();
        let mut fetched = 0u32;
        unsafe {
            let _ = enum_var.Next(
                std::slice::from_mut(&mut variant),
                &raw mut fetched,
            );
        }
        if fetched == 0 {
            break;
        }

        let Ok(com_rule) = (unsafe { extract_rule_from_variant(&variant) }) else {
            continue;
        };

        let mut fw_rule = unsafe { read_rule_state(&com_rule) };

        if let Some(f) = filter
            && !matches_filter(&fw_rule, f)
        {
            continue;
        }

        // Avoid duplicates names for the canonical name
        if let Some(name) = &fw_rule.name {
            // if name exists, increment the count
            if names.contains_key(name) {
                *names.get_mut(name).unwrap() += 1;
                fw_rule.canonical_name = Some(format!("{} ({})", name, names[name]));
                results.push(fw_rule.clone());
                continue;
            }
            fw_rule.canonical_name = Some(name.clone());
            results.push(fw_rule.clone());
            names.insert(name.clone(), 0);
        }
    }

    Ok(results)
}
