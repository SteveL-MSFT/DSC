// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, PartialEq, Eq, Clone, ValueEnum)]
pub enum ServiceType {
    Driver,
    Service,
}

#[derive(Parser)]
#[clap(name = "servicedsc", version = "0.1.0", about = "Manage Windows drivers and services", long_about = None)]
pub struct Arguments {
    #[clap(subcommand)]
    pub subcommand: SubCommand,
    #[clap(short, long, default_value = "driver", help = "The service type.")]
    pub service_type: ServiceType,
}

#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum SubCommand {
    #[clap(name = "get", about = "Retrieve service/driver configuration.")]
    Get {
        #[clap(short, long, required = true, help = "The JSON input.")]
        input: String,
    },
    #[clap(name = "set", about = "Apply service/driver configuration.")]
    Set {
        #[clap(short, long, required = true, help = "The JSON input.")]
        input: String,
    },
    #[clap(name = "export", about = "Export service/driver configuration.")]
    Export {
        #[clap(short, long, help = "The JSON input.")]
        input: Option<String>,
    },
    #[clap(name = "schema", about = "Retrieve JSON schema.")]
    Schema,
}
