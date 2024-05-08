use clap::Args;
use log::{debug, info};
use std::error::Error;
use std::io::Write;

use crate::cli::GlobalOptions;
use row::cluster;

#[derive(Args, Debug)]
pub struct Arguments {
    /// Show all clusters.
    #[arg(long, group = "select", display_order = 0)]
    all: bool,

    /// Show only the autodetected cluster's name.
    #[arg(long, group = "select", display_order = 0)]
    name: bool,
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
        info!("All cluster configurations:");
        write!(output, "{}", &toml::to_string_pretty(&clusters)?)?;
    } else {
        let cluster = clusters.identify(options.cluster.as_deref())?;
        info!("Cluster configurations for '{}':", cluster.name);

        if args.name {
            writeln!(output, "{}", cluster.name)?;
        } else {
            write!(output, "{}", &toml::to_string_pretty(&cluster)?)?;
        }
    }

    Ok(())
}
