// Copyright (c) 2024 The Regents of the University of Michigan.
// Part of row, released under the BSD 3-Clause License.

use clap::Args;
use log::{debug, info};
use std::error::Error;
use std::io::Write;

use crate::cli::GlobalOptions;
use row::cluster;

#[derive(Args, Debug)]
pub struct Arguments {
    /// Show all clusters.
    #[arg(long, display_order = 0)]
    all: bool,

    /// Show only the cluster name(s).
    #[arg(long, display_order = 0)]
    short: bool,
}

/// Show the cluster.
///
/// Print the cluster to stdout in toml format.
///
pub fn cluster<W: Write>(
    options: &GlobalOptions,
    args: &Arguments,
    output: &mut W,
) -> Result<(), Box<dyn Error>> {
    debug!("Showing clusters.");

    let clusters = cluster::Configuration::open()?;

    if args.all {
        if args.short {
            for cluster in clusters.cluster {
                writeln!(output, "{}", cluster.name)?;
            }
        } else {
            info!("All cluster configurations:");
            write!(output, "{}", &toml::to_string_pretty(&clusters)?)?;
        }
    } else {
        let cluster = clusters.identify(options.cluster.as_deref())?;
        info!("Cluster configurations for '{}':", cluster.name);

        if args.short {
            writeln!(output, "{}", cluster.name)?;
        } else {
            write!(output, "{}", &toml::to_string_pretty(&cluster)?)?;
        }
    }

    Ok(())
}
