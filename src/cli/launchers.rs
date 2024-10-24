// Copyright (c) 2024 The Regents of the University of Michigan.
// Part of row, released under the BSD 3-Clause License.

use clap::Args;
use log::{debug, info};
use std::error::Error;
use std::io::Write;

use crate::cli::GlobalOptions;
use row::cluster;
use row::launcher;

#[derive(Args, Debug)]
pub struct Arguments {
    /// Show all launchers.
    #[arg(long, display_order = 0)]
    all: bool,

    /// Show only launcher names.
    #[arg(long, display_order = 0, conflicts_with = "all")]
    short: bool,
}

/// Show the launchers.
///
/// Print the launchers to stdout in toml format.
///
pub fn launchers<W: Write>(
    options: &GlobalOptions,
    args: &Arguments,
    output: &mut W,
) -> Result<(), Box<dyn Error>> {
    debug!("Showing launchers.");

    let launchers = launcher::Configuration::open()?;

    if args.all {
        info!("All launcher configurations:");
        write!(
            output,
            "{}",
            &toml::to_string_pretty(launchers.full_config())?
        )?;
    } else {
        let clusters = cluster::Configuration::open()?;
        let cluster = clusters.identify(options.cluster.as_deref())?;

        if args.short {
            for launcher_name in launchers.by_cluster(&cluster.name).keys() {
                writeln!(output, "{launcher_name}")?;
            }
        } else {
            info!("Launcher configurations for cluster '{}':", cluster.name);
            write!(
                output,
                "{}",
                &toml::to_string_pretty(&launchers.by_cluster(&cluster.name))?
            )?;
        }
    }

    Ok(())
}
