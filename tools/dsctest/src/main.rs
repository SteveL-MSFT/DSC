// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod args;
mod delete;
mod exist;
mod exit_code;
mod export;
mod exporter;
mod in_desired_state;
mod metadata;
mod sleep;
mod trace;
mod whatif;

use args::{Args, Schemas, SubCommand};
use clap::Parser;
use schemars::schema_for;
use serde_json::Map;
use crate::delete::Delete;
use crate::exist::{Exist, State};
use crate::exit_code::ExitCode;
use crate::export::Export;
use crate::exporter::{Exporter, Resource};
use crate::in_desired_state::InDesiredState;
use crate::metadata::Metadata;
use crate::sleep::Sleep;
use crate::trace::Trace;
use crate::whatif::WhatIf;
use std::{thread, time::Duration};

#[allow(clippy::too_many_lines)]
fn main() {
    let args = Args::parse();
    let json = match args.subcommand {
        SubCommand::Delete { input } => {
            let mut delete = match serde_json::from_str::<Delete>(&input) {
                Ok(delete) => delete,
                Err(err) => {
                    eprintln!("Error JSON does not match schema: {err}");
                    std::process::exit(1);
                }
            };
            delete.delete_called = Some(true);
            serde_json::to_string(&delete).unwrap()
        },
        SubCommand::Exist { input } => {
            let mut exist = match serde_json::from_str::<Exist>(&input) {
                Ok(exist) => exist,
                Err(err) => {
                    eprintln!("Error JSON does not match schema: {err}");
                    std::process::exit(1);
                }
            };
            if exist.exist {
                exist.state = Some(State::Present);
            } else {
                exist.state = Some(State::Absent);
            }

            serde_json::to_string(&exist).unwrap()
        },
        SubCommand::ExitCode { input } => {
            let exit_code = match serde_json::from_str::<ExitCode>(&input) {
                Ok(exit_code) => exit_code,
                Err(err) => {
                    eprintln!("Error JSON does not match schema: {err}");
                    std::process::exit(1);
                }
            };
            if exit_code.exit_code != 0 {
                eprintln!("Exiting with code: {}", exit_code.exit_code);
                std::process::exit(exit_code.exit_code);
            }
            input
        },
        SubCommand::Export { input } => {
            let export = match serde_json::from_str::<Export>(&input) {
                Ok(export) => export,
                Err(err) => {
                    eprintln!("Error JSON does not match schema: {err}");
                    std::process::exit(1);
                }
            };
            for i in 0..export.count {
                let instance = Export {
                    count: i,
                    _kind: Some("TestKind".to_string()),
                    _name: Some("TestName".to_string()),
                    _security_context: Some("elevated".to_string()),
                };
                println!("{}", serde_json::to_string(&instance).unwrap());
            }
            String::new()
        },
        SubCommand::Exporter { input } => {
            let exporter = match serde_json::from_str::<Exporter>(&input) {
                Ok(exporter) => exporter,
                Err(err) => {
                    eprintln!("Error JSON does not match schema: {err}");
                    std::process::exit(1);
                }
            };
            for type_name in exporter.type_names {
                let mut resource = Resource {
                    name: "test".to_string(),
                    r#type: type_name,
                    properties: Map::new(),
                };
                resource.properties.insert("foo".to_string(), serde_json::Value::String("bar".to_string()));
                resource.properties.insert("hello".to_string(), serde_json::Value::String("world".to_string()));
                println!("{}", serde_json::to_string(&resource).unwrap());
            }
            String::new()
        },
        SubCommand::InDesiredState { input } => {
            let mut in_desired_state = match serde_json::from_str::<in_desired_state::InDesiredState>(&input) {
                Ok(in_desired_state) => in_desired_state,
                Err(err) => {
                    eprintln!("Error JSON does not match schema: {err}");
                    std::process::exit(1);
                }
            };
            in_desired_state.value_one = 1;
            in_desired_state.value_two = 2;
            serde_json::to_string(&in_desired_state).unwrap()
        },
        SubCommand::Metadata { input, export } => {
            let count = if export {
                3
            } else {
                1
            };
            for i in 0..count {
                let mut metadata = match serde_json::from_str::<Metadata>(&input) {
                    Ok(metadata) => metadata,
                    Err(err) => {
                        eprintln!("Error JSON does not match schema: {err}");
                        std::process::exit(1);
                    }
                };
                metadata.name = Some(format!("Metadata example {}", i+1));
                metadata.count = Some(i + 1);
                println!("{}", serde_json::to_string(&metadata).unwrap());
            }
            String::new()
        },
        SubCommand::Schema { subcommand } => {
            let schema = match subcommand {
                Schemas::Delete => {
                    schema_for!(Delete)
                },
                Schemas::Exist => {
                    schema_for!(Exist)
                },
                Schemas::ExitCode => {
                    schema_for!(ExitCode)
                },
                Schemas::Export => {
                    schema_for!(Export)
                },
                Schemas::Exporter => {
                    schema_for!(Exporter)
                },
                Schemas::InDesiredState => {
                    schema_for!(InDesiredState)
                },
                Schemas::Metadata => {
                    schema_for!(Metadata)
                },
                Schemas::Sleep => {
                    schema_for!(Sleep)
                },
                Schemas::Trace => {
                    schema_for!(Trace)
                },
                Schemas::WhatIf => {
                    schema_for!(WhatIf)
                },
            };
            serde_json::to_string(&schema).unwrap()
        },
        SubCommand::Sleep { input } => {
            let sleep = match serde_json::from_str::<Sleep>(&input) {
                Ok(sleep) => sleep,
                Err(err) => {
                    eprintln!("Error JSON does not match schema: {err}");
                    std::process::exit(1);
                }
            };
            thread::sleep(Duration::from_secs(sleep.seconds));
            serde_json::to_string(&sleep).unwrap()
        },
        SubCommand::Trace => {
            // get level from DSC_TRACE_LEVEL env var
            let level = match std::env::var("DSC_TRACE_LEVEL") {
                Ok(level) => level,
                Err(_) => "warn".to_string(),
            };
            let trace = trace::Trace {
                level,
            };
            serde_json::to_string(&trace).unwrap()
        },
        SubCommand::WhatIf { what_if } => {
            let result: WhatIf = if what_if {
                WhatIf { execution_type: "WhatIf".to_string() }
            } else {
                WhatIf { execution_type: "Actual".to_string() }
            };
            serde_json::to_string(&result).unwrap()
        },
    };

    if !json.is_empty() {
        println!("{json}");
    }
}
