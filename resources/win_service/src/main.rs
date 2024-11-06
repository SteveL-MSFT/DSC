mod args;
mod config;

use args::{Arguments, ServiceType, SubCommand};
use clap::Parser;
use config::{Service, StartupType, ErrorControl};
use registry::{Data, Hive, RegKey, Security};
use schemars::schema_for;
use std::{path::Path, process::exit};
use tracing::{debug, error};
use tracing_subscriber::{filter::LevelFilter, prelude::__tracing_subscriber_SubscriberExt, EnvFilter, Layer};

const EXIT_INVALID_INPUT: i32 = 1;
const EXIT_REGISTRY_ERROR: i32 = 2;

fn main() {
    enable_tracing();

    let args = Arguments::parse();
    match args.subcommand {
        SubCommand::Get { input } => {
            debug!("Get input: {input}");
        },
        SubCommand::Set { input } => {
            debug!("Set input: {input}");
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
                export(args.service_type, &Some(service));
            } else {
                export(args.service_type, &None);
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

fn open_registry(system_root: &Option<String>, permission: Security) -> RegKey {
    const SERVICES_REG_PATH: &str =r#"SYSTEM\CurrentControlSet\Services"#;
    if let Some(system_root) = system_root {
        match Hive::load_file(
            Path::new(format!("{system_root}\\Windows\\System32\\config\\SYSTEM").as_str()),
            permission,
        ) {
            Ok(hive) => match hive.open(SERVICES_REG_PATH, permission) {
                Ok(reg_key) => reg_key,
                Err(err) => {
                    error!("Could not open reg hive: {err}");
                    exit(EXIT_REGISTRY_ERROR);
                }
            },
            Err(err) => {
                error!("Could not open reg file: {err}");
                exit(EXIT_REGISTRY_ERROR);
            }
        }
    } else {
        match Hive::LocalMachine.open(SERVICES_REG_PATH, permission) {
            Ok(reg_key) => reg_key,
            Err(err) => {
                error!("Could not open reg hive: {err}");
                exit(EXIT_REGISTRY_ERROR);
            }
        }
    }
}

fn export(service_type: ServiceType, service_input: &Option<Service>) {
    let reg_key = if let Some(service_input) = service_input {
        open_registry(&service_input.system_root, Security::Read)
    } else {
        open_registry(&None, Security::Read)
    };
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
            match service_reg_type {
                Data::U32(reg_service_type) => {
                    service.service_type = match reg_service_type {
                        1 => {
                            if service_type != ServiceType::Driver {
                                continue;
                            }
                            config::ServiceType::KernelDriver
                        },
                        2 => {
                            if service_type != ServiceType::Driver {
                                continue;
                            }
                            config::ServiceType::FileSystemDriver
                        },
                        16 => {
                            if service_type != ServiceType::Service {
                                continue;
                            }
                            config::ServiceType::Win32OwnProcess
                        },
                        32 => {
                            if service_type != ServiceType::Service {
                                continue;
                            }
                            config::ServiceType::Win32ShareProcess
                        },
                        _ => {
                            debug!("Unknown service type: {reg_service_type} for {:?}", subkey);
                            continue;
                        }
                    };
                },
                _ => {
                    debug!("Invalid service type: {service_reg_type} for {:?}", subkey);
                    continue;
                },
            }
        } else {
            debug!("Service type not found: {:?}", subkey);
            continue;
        }
        if let Ok(display_name) = key.value("DisplayName") {
            // TODO: extract localized display name
            service.display_name = display_name.to_string();
        }
        if let Ok(image_path) = key.value("ImagePath") {
            service.image_path = image_path.to_string();
        }
        if let Ok(description) = key.value("Description") {
            // TODO: extract localized description
            service.description = Some(description.to_string());
        }
        if let Ok(start_type) = key.value("Start") {
            match start_type {
                Data::U32(start_type) => {
                    service.start_type = match start_type {
                        0 => StartupType::Boot,
                        1 => StartupType::System,
                        2 => StartupType::Automatic,
                        3 => StartupType::Demand,
                        4 => StartupType::Disabled,
                        _ => {
                            debug!("Unknown start type: {start_type} for {:?}", subkey);
                            continue;
                        },
                    };
                },
                _ => {
                    debug!("Invalid start type: {start_type} for {:?}", subkey);
                    continue;
                },
            }
        } else {
            debug!("Start type not found: {:?}", subkey);
            continue;
        }
        if let Ok(error_control) = key.value("ErrorControl") {
            match error_control {
                Data::U32(error_control) => {
                    service.error_control = match error_control {
                        0 => ErrorControl::Ignore,
                        1 => ErrorControl::Normal,
                        2 => ErrorControl::Severe,
                        3 => ErrorControl::Critical,
                        _ => {
                            debug!("Unknown error control: {error_control} for {:?}", subkey);
                            continue;
                        },
                    };
                },
                _ => {
                    debug!("Invalid error control: {error_control} for {:?}", subkey);
                    continue;
                },
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
