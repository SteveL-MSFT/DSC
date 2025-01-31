mod args;
mod config;

use args::{Arguments, ServiceType, SubCommand};
use clap::Parser;
use config::Service;
use offreg::{hive::OfflineRegistryHive, key::{OfflineRegistryKey, OfflineRegistryValueData}};
use registry::{Data, Hive, RegKey, Security};
use schemars::schema_for;
use std::process::exit;
use tracing::{debug, trace, error};
use tracing_subscriber::{filter::LevelFilter, prelude::__tracing_subscriber_SubscriberExt, EnvFilter, Layer};
const EXIT_INVALID_INPUT: i32 = 1;
const EXIT_REGISTRY_ERROR: i32 = 2;
const SERVICES_REG_PATH: &str = r"SYSTEM\CurrentControlSet\Services";
const OFFLINE_SERVICES_REG_PATH: &str = r"ControlSet001\Services";

fn main() {
    enable_tracing();

    let args = Arguments::parse();
    match args.subcommand {
        SubCommand::Get { input } => {
            println!("{input}");
        },
        SubCommand::Set { input } => {
            trace!("Set input: {input}");
            let service = match serde_json::from_str::<Service>(&input) {
                Ok(service) => service,
                Err(err) => {
                    error!("{err}");
                    exit(EXIT_INVALID_INPUT);
                }
            };
            debug!("Set input: {service:?}");
            set(&args.service_type, &service);
        },
        SubCommand::Export { input } => {
            if let Some(input) = input {
                let service = match serde_json::from_str::<Service>(&input) {
                    Ok(service) => service,
                    Err(err) => {
                        error!("{err}");
                        exit(EXIT_INVALID_INPUT);
                    }
                };
                export(&args.service_type, Some(&service));
            } else {
                export(&args.service_type, None);
            }
        },
        SubCommand::Schema => {
            debug!("Schema");
            let schema = schema_for!(Service);
            let json = serde_json::to_string(&schema).unwrap();
            println!("{json}");
        },
    }
}

fn open_offline_registry(system_root: &str) -> OfflineRegistryKey {
    let reg_file_path = format!("{system_root}\\Windows\\System32\\config\\SYSTEM");
    debug!("Opening offline reg file hive: {reg_file_path}");
    let mut hive = OfflineRegistryHive::new();
    match hive.load(&reg_file_path) {
        Ok(()) => {
            match hive.open_key(OFFLINE_SERVICES_REG_PATH) {
                Ok(key) => key,
                Err(err) => {
                    error!("{err}");
                    exit(EXIT_REGISTRY_ERROR);
                }
            }
        },
        Err(err) => {
            error!("Could not open reg hive: {err}");
            exit(EXIT_REGISTRY_ERROR);
        }
    }
}

fn open_registry(permission: Security) -> RegKey {
    match Hive::LocalMachine.open(SERVICES_REG_PATH, permission) {
        Ok(reg_key) => reg_key,
        Err(err) => {
            error!("Could not open reg hive: {err}");
            exit(EXIT_REGISTRY_ERROR);
        }
    }
}

fn get(service_type: &ServiceType, service: &Service) {
    if let Some(system_root) = std::env::var("SYSTEMROOT").ok() {
        // only use offline registry if the system root is different from the current system root
        if system_root.to_lowercase() != std::env::var("SYSTEMROOT").unwrap().to_lowercase() {
            let reg_key = open_offline_registry(system_root.as_str());
            let service_key = match reg_key.open_subkey(service.name.as_str()) {
                Ok(service_key) => service_key,
                Err(err) => {
                    error!("Failed to open service {}: {err}", service.name);
                    exit(EXIT_REGISTRY_ERROR);
                }
            };
            let service = get_offline_registry(service_type, service, &service_key);
            let json = match serde_json::to_string(&service) {
                Ok(json) => json,
                Err(err) => {
                    error!("{err}");
                    exit(EXIT_REGISTRY_ERROR);
                }
            };
            println!("{json}");
            return;
        }
    }

    error!("Online registry not supported for getting services");
}

fn get_offline_registry(service_type: &ServiceType, service: &Service, reg_key: &OfflineRegistryKey) -> Service {
    let mut service = Service::default();
    if let Ok(service_reg_type) = reg_key.get_value("Type") {
        if let Some(reg_data) = service_reg_type.data {
            if let OfflineRegistryValueData::Dword(service_reg_type) = reg_data {
                if let Some(converted_service_type) = convert_service_type(service_reg_type) {
                    match service_type {
                        ServiceType::Driver => {
                            if converted_service_type == config::ServiceType::KernelDriver || converted_service_type == config::ServiceType::FileSystemDriver {
                                service.service_type = Some(converted_service_type);
                            } else {
                                error!("Invalid service type for driver: {service:?}");
                                exit(EXIT_INVALID_INPUT);
                            }
                        },
                        ServiceType::Service => {
                            if converted_service_type == config::ServiceType::Win32OwnProcess || converted_service_type == config::ServiceType::Win32ShareProcess {
                                service.service_type = Some(converted_service_type);
                            } else {
                                error!("Invalid service type for service: {service:?}");
                                exit(EXIT_INVALID_INPUT);
                            }
                        }
                    }
                }
            } else {
                error!("Invalid service type: {service:?}");
                exit(EXIT_INVALID_INPUT);
            }
        } else {
            error!("Missing service type: {service:?}");
            exit(EXIT_INVALID_INPUT);
        }
    } else {
        error!("Service type not found: {service:?}");
        exit(EXIT_INVALID_INPUT);
    }
    if let Ok(display_name) = reg_key.get_value("DisplayName") {
        if let Some(reg_data) = display_name.data {
            service.display_name = Some(reg_data.to_string());
        }
    }
    if let Ok(image_path) = reg_key.get_value("ImagePath") {
        if let Some(reg_data) = image_path.data {
            service.image_path = Some(reg_data.to_string());
        }
    }
    if let Ok(description) = reg_key.get_value("Description") {
        if let Some(reg_data) = description.data {
            service.description = Some(reg_data.to_string());
        }
    }
    if let Ok(start_type) = reg_key.get_value("Start") {
        if let Some(reg_data) = start_type.data {
            if let OfflineRegistryValueData::Dword(start_type) = reg_data {
                service.start_type = convert_start_type(start_type);
            } else {
                error!("Invalid start type: {service:?}");
                exit(EXIT_INVALID_INPUT);
            }
        } else {
            error!("Invalid start type: {service:?}");
            exit(EXIT_INVALID_INPUT);
        }
    } else {
        error!("Start type not found: {service:?}");
        exit(EXIT_INVALID_INPUT);
    }
    if let Ok(error_control) = reg_key.get_value("ErrorControl") {
        if let Some(reg_data) = error_control.data {
            if let OfflineRegistryValueData::Dword(error_control) = reg_data {
                service.error_control = convert_error_control(error_control);
            } else {
                error!("Invalid error control: {service:?}");
                exit(EXIT_INVALID_INPUT);
            }
        } else {
            error!("Invalid error control: {service:?}");
            exit(EXIT_INVALID_INPUT);
        }
    } else {
        error!("Error control not found: {service:?}");
        exit(EXIT_INVALID_INPUT);
    }
    service
}

fn set(service_type: &ServiceType, service: &Service) {
    if let Some(system_root) = &service.system_root {
        // only use offline registry if the system root is different from the current system root
        if let Some(system_drive) = std::env::var("SYSTEMDRIVE").ok() {
            if system_drive.to_lowercase() != system_root.to_lowercase() {
                let reg_key = open_offline_registry(system_root);
                let service_key = match reg_key.open_subkey(service.name.as_str()) {
                    Ok(service_key) => service_key,
                    Err(err) => {
                        error!("Failed to open service {}: {err}", service.name);
                        exit(EXIT_REGISTRY_ERROR);
                    }
                };
                set_offline_registry(service_type, service, &service_key);
                let json = match serde_json::to_string(&service) {
                    Ok(json) => json,
                    Err(err) => {
                        error!("{err}");
                        exit(EXIT_REGISTRY_ERROR);
                    }
                };
                println!("{json}");
                return;
            }
        }

        error!("Online registry not supported for setting services");
    }
}

fn set_offline_registry(service_type: &ServiceType, service: &Service, reg_key: &OfflineRegistryKey) {
    debug!("Setting {service_type:?}: {service:?}");
    if let Err(err) = reg_key.set_value("Start", OfflineRegistryValueData::Dword(convert_start_type_to_value(service.start_type.as_ref().unwrap()))) {
        error!("Failed to set start type: {err}");
        exit(EXIT_REGISTRY_ERROR);
    }
}

fn export(service_type: &ServiceType, service_input: Option<&Service>) {
    if let Some(service_input) = service_input {
        if let Some(system_root) = &service_input.system_root {
            // only use offline registry if the system root is different from the current system root
            if let Some(system_drive) = std::env::var("SYSTEMDRIVE").ok() {
                if system_drive.to_lowercase() != system_root.to_lowercase() {
                    let reg_key = open_offline_registry(system_root);
                    enum_offline_registry(service_type, &reg_key);
                    return;
                }
            }
        }
    }

    let reg_key = open_registry(Security::Read);
    enum_registry(service_type, &reg_key);
}

#[allow(clippy::too_many_lines)]
fn enum_offline_registry(service_type: &ServiceType, reg_key: &OfflineRegistryKey) {
    let mut service = Service::default();
    let sub_keys = match reg_key.enumerate_subkeys() {
        Ok(sub_keys) => sub_keys,
        Err(err) => {
            error!("{err}");
            exit(EXIT_REGISTRY_ERROR);
        }
    };
    for subkey in sub_keys {
        service.name = subkey.to_string();
        if let Ok(service_reg_type) = subkey.get_value("Type") {
            if let Some(reg_data) = service_reg_type.data {
                    if let OfflineRegistryValueData::Dword(service_reg_type) = reg_data {
                        if let Some(converted_service_type) = convert_service_type(service_reg_type) {
                            match service_type {
                                ServiceType::Driver => {
                                    if converted_service_type == config::ServiceType::KernelDriver || converted_service_type == config::ServiceType::FileSystemDriver {
                                        service.service_type = Some(converted_service_type);
                                    } else {
                                        continue;
                                    }
                                },
                                ServiceType::Service => {
                                    if converted_service_type == config::ServiceType::Win32OwnProcess || converted_service_type == config::ServiceType::Win32ShareProcess {
                                        service.service_type = Some(converted_service_type);
                                    } else {
                                        continue;
                                    }
                                }
                            }
                        }
                    } else {
                        debug!("Invalid service type: {subkey}");
                        continue;
                    }            
                } else {
                debug!("Missing service type: {subkey}");
                continue;
            }
        } else {
            debug!("Service type not found: {subkey}");
            continue;
        }
        if let Ok(display_name) = subkey.get_value("DisplayName") {
            if let Some(reg_data) = display_name.data {
                service.display_name = Some(reg_data.to_string());
            }
        }
        if let Ok(image_path) = subkey.get_value("ImagePath") {
            if let Some(reg_data) = image_path.data {
                service.image_path = Some(reg_data.to_string());
            }
        }
        if let Ok(description) = subkey.get_value("Description") {
            if let Some(reg_data) = description.data {
                service.description = Some(reg_data.to_string());
            }
        }
        if let Ok(start_type) = subkey.get_value("Start") {
            if let Some(reg_data) = start_type.data {
                if let OfflineRegistryValueData::Dword(start_type) = reg_data {
                    service.start_type = convert_start_type(start_type);
                } else {
                    debug!("Invalid start type: {subkey}");
                    continue;
                }
            } else {
                debug!("Invalid start type: {subkey}");
                continue;
            }
        } else {
            debug!("Start type not found: {subkey}");
            continue;
        }
        if let Ok(error_control) = subkey.get_value("ErrorControl") {
            if let Some(reg_data) = error_control.data {
                if let OfflineRegistryValueData::Dword(error_control) = reg_data {
                    service.error_control = convert_error_control(error_control);
                } else {
                    debug!("Invalid error control: {subkey}");
                    continue;
                }
            } else {
                debug!("Invalid error control: {subkey}");
                continue;
            }
        } else {
            debug!("Error control not found: {subkey}");
            continue;
        }
        // write service to stdout
        let json = match serde_json::to_string(&service) {
            Ok(json) => json,
            Err(err) => {
                error!("{err}");
                exit(EXIT_REGISTRY_ERROR);
            }
        };
        println!("{json}");
    }
}

fn convert_service_type(service_type: u32) -> Option<config::ServiceType> {
    match service_type {
        1 => Some(config::ServiceType::KernelDriver),
        2 => Some(config::ServiceType::FileSystemDriver),
        16 => Some(config::ServiceType::Win32OwnProcess),
        32 => Some(config::ServiceType::Win32ShareProcess),
        _ => {
            debug!("Unknown service type: {service_type}");
            None
        },
    }
}

fn convert_start_type(start_type: u32) -> Option<config::StartupType> {
    match start_type {
        0 => Some(config::StartupType::Boot),
        1 => Some(config::StartupType::System),
        2 => Some(config::StartupType::Automatic),
        3 => Some(config::StartupType::Demand),
        4 => Some(config::StartupType::Disabled),
        _ => {
            debug!("Unknown start type: {start_type}");
            None
        },
    }
}

fn convert_start_type_to_value(start_type: &config::StartupType) -> u32 {
    match start_type {
        config::StartupType::Boot => 0,
        config::StartupType::System => 1,
        config::StartupType::Automatic => 2,
        config::StartupType::Demand => 3,
        config::StartupType::Disabled => 4,
        config::StartupType::Unknown => {
            debug!("Unknown start type: {start_type:?}");
            0 // this function should return a Result so it can error instead of returning 0
        }
    }
}

fn convert_error_control(error_control: u32) -> Option<config::ErrorControl> {
    match error_control {
        0 => Some(config::ErrorControl::Ignore),
        1 => Some(config::ErrorControl::Normal),
        2 => Some(config::ErrorControl::Severe),
        3 => Some(config::ErrorControl::Critical),
        _ => {
            debug!("Unknown error control: {error_control}");
            None
        },
    }
}

fn enum_registry(service_type: &ServiceType, reg_key: &RegKey) {
    let mut service = Service::default();
    for subkey in reg_key.keys() {
        // TODO: handle filtering
        let subkey = match subkey {
            Ok(subkey) => subkey,
            Err(err) => {
                debug!("{err}");
                continue;
            }
        };
        service.name = subkey.to_string();
        let key = match subkey.open(Security::Read) {
            Ok(key) => key,
            Err(err) => {
                error!("{err}");
                exit(EXIT_REGISTRY_ERROR);
            }
        };
        if let Ok(service_reg_type) = key.value("Type") {
            if let Data::U32(service_reg_type) = service_reg_type {
                if let Some(converted_service_type) = convert_service_type(service_reg_type) {
                    match service_type {
                        ServiceType::Driver => {
                            if converted_service_type == config::ServiceType::KernelDriver || converted_service_type == config::ServiceType::FileSystemDriver {
                                service.service_type = Some(converted_service_type);
                            } else {
                                continue;
                            }
                        },
                        ServiceType::Service => {
                            if converted_service_type == config::ServiceType::Win32OwnProcess || converted_service_type == config::ServiceType::Win32ShareProcess {
                                service.service_type = Some(converted_service_type);
                            } else {
                                continue;
                            }
                        }
                    }
                }
            } else {
                debug!("Invalid service type: {:?}", subkey);
                continue;
            }
        } else {
            debug!("Service type not found: {:?}", subkey);
            continue;
        }
        if let Ok(display_name) = key.value("DisplayName") {
            // TODO: extract localized display name
            service.display_name = Some(display_name.to_string());
        }
        if let Ok(image_path) = key.value("ImagePath") {
            service.image_path = Some(image_path.to_string());
        }
        if let Ok(description) = key.value("Description") {
            // TODO: extract localized description
            service.description = Some(description.to_string());
        }
        if let Ok(start_type) = key.value("Start") {
            if let Data::U32(start_type) = start_type {
                service.start_type = convert_start_type(start_type);
            } else {
                debug!("Invalid start type: {:?}", subkey);
                continue;
            }
        } else {
            debug!("Start type not found: {:?}", subkey);
            continue;
        }
        if let Ok(error_control) = key.value("ErrorControl") {
            if let Data::U32(error_control) = error_control {
                service.error_control = convert_error_control(error_control);
            } else {
                debug!("Invalid error control: {:?}", subkey);
                continue;
            }
        } else {
            debug!("Error control not found: {:?}", subkey);
            continue;
        }
        // write service to stdout
        let json = match serde_json::to_string(&service) {
            Ok(json) => json,
            Err(err) => {
                error!("{err}");
                exit(EXIT_REGISTRY_ERROR);
            }
        };
        println!("{json}");
    }
}

fn enable_tracing() {
    // default filter to trace level
    let filter = EnvFilter::builder().with_default_directive(LevelFilter::TRACE.into()).parse("").unwrap_or_default();
    let layer = tracing_subscriber::fmt::Layer::default().with_writer(std::io::stderr);
    let fmt = layer
        .with_ansi(false)
        .with_level(true)
        .with_line_number(true)
        .json()
        .boxed();

    let subscriber = tracing_subscriber::Registry::default().with(fmt).with(filter);

    if tracing::subscriber::set_global_default(subscriber).is_err() {
        eprintln!("Unable to set global default tracing subscriber.  Tracing is diabled.");
    }
}
