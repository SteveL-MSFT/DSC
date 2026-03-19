// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use serde::{Deserialize, Serialize};

/// The direction of network traffic a firewall rule applies to.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum Direction {
    Inbound,
    Outbound,
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::Inbound => write!(f, "Inbound"),
            Direction::Outbound => write!(f, "Outbound"),
        }
    }
}

/// The action to take when a firewall rule matches traffic.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum Action {
    Allow,
    Block,
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Allow => write!(f, "Allow"),
            Action::Block => write!(f, "Block"),
        }
    }
}

/// The network protocol a firewall rule applies to.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[allow(clippy::upper_case_acronyms)]
pub enum Protocol {
    TCP,
    UDP,
    ICMPv4,
    ICMPv6,
    Any,
}

impl Protocol {
    /// Convert to the Windows Firewall COM API protocol number.
    #[must_use]
    pub fn to_protocol_number(&self) -> i32 {
        match self {
            Protocol::TCP => 6,
            Protocol::UDP => 17,
            Protocol::ICMPv4 => 1,
            Protocol::ICMPv6 => 58,
            Protocol::Any => 256,
        }
    }

    /// Convert from a Windows Firewall COM API protocol number.
    #[must_use]
    pub fn from_protocol_number(n: i32) -> Option<Self> {
        match n {
            6 => Some(Protocol::TCP),
            17 => Some(Protocol::UDP),
            1 => Some(Protocol::ICMPv4),
            58 => Some(Protocol::ICMPv6),
            256 => Some(Protocol::Any),
            _ => None,
        }
    }
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Protocol::TCP => write!(f, "TCP"),
            Protocol::UDP => write!(f, "UDP"),
            Protocol::ICMPv4 => write!(f, "ICMPv4"),
            Protocol::ICMPv6 => write!(f, "ICMPv6"),
            Protocol::Any => write!(f, "Any"),
        }
    }
}

/// A network profile that a firewall rule can apply to.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum Profile {
    Domain,
    Private,
    Public,
}

impl Profile {
    /// Convert profile bitmask from the COM API to a list of profiles.
    #[must_use]
    pub fn from_bitmask(mask: i32) -> Vec<Self> {
        // NET_FW_PROFILE2_ALL = 0x7FFFFFFF
        if mask == 0x7FFF_FFFF {
            return vec![Profile::Domain, Profile::Private, Profile::Public];
        }
        let mut result = Vec::new();
        if mask & 0x1 != 0 {
            result.push(Profile::Domain);
        }
        if mask & 0x2 != 0 {
            result.push(Profile::Private);
        }
        if mask & 0x4 != 0 {
            result.push(Profile::Public);
        }
        result
    }

    /// Convert a list of profiles to a bitmask for the COM API.
    #[must_use]
    pub fn to_bitmask(profiles: &[Self]) -> i32 {
        profiles.iter().fold(0i32, |acc, p| {
            acc | match p {
                Profile::Domain => 0x1,
                Profile::Private => 0x2,
                Profile::Public => 0x4,
            }
        })
    }
}

impl std::fmt::Display for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Profile::Domain => write!(f, "Domain"),
            Profile::Private => write!(f, "Private"),
            Profile::Public => write!(f, "Public"),
        }
    }
}

/// Represents a Windows Firewall rule resource.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FirewallRule {
    /// The name of the firewall rule.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The canonical name of the firewall rule (read-only).
    #[serde(rename = "_name", skip_serializing_if = "Option::is_none")]
    pub canonical_name: Option<String>,

    /// Whether the firewall rule exists.
    #[serde(rename = "_exist", skip_serializing_if = "Option::is_none")]
    pub exist: Option<bool>,

    /// A description of the firewall rule.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Whether the firewall rule is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// The direction of network traffic the rule applies to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<Direction>,

    /// The action to take when the rule matches.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<Action>,

    /// The network protocol the rule applies to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<Protocol>,

    /// The local ports the rule applies to (comma-separated).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_ports: Option<String>,

    /// The remote ports the rule applies to (comma-separated).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_ports: Option<String>,

    /// The local addresses the rule applies to (comma-separated).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_addresses: Option<String>,

    /// The remote addresses the rule applies to (comma-separated).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_addresses: Option<String>,

    /// The network profiles the rule applies to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profiles: Option<Vec<Profile>>,

    /// The full path to the program the rule applies to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub program: Option<String>,
}

/// Represents an error from a Windows Firewall operation.
#[derive(Debug)]
pub struct FirewallError {
    pub message: String,
}

impl std::fmt::Display for FirewallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for FirewallError {}

impl From<String> for FirewallError {
    fn from(message: String) -> Self {
        Self { message }
    }
}
