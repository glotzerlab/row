use clap::Args;
use log::{debug, info};
use std::error::Error;
use std::io::Write;

use crate::cli::GlobalOptions;
use row::cluster::ClusterConfiguration;
use row::launcher::LauncherConfiguration;

#[derive(Args, Debug)]
pub struct LaunchersArgs {
    /// Show all launchers.
    #[arg(long, display_order = 0)]
    all: bool,
}

/// Show the launchers.
///
/// Print the launchers to stdout in toml format.
///
pub fn launchers<W: Write>(
    options: GlobalOptions,
    args: LaunchersArgs,
    output: &mut W,
) -> Result<(), Box<dyn Error>> {
    debug!("Showing launchers.");

    let launchers = LauncherConfiguration::open()?;

    if args.all {
        info!("All launcher configurations:");
        write!(
            output,
            "{}",
            &toml::to_string_pretty(launchers.full_config())?
        )?;
    } else {
        let clusters = ClusterConfiguration::open()?;
        let cluster = clusters.identify(options.cluster.as_deref())?;

        info!("Launcher configurations for cluster '{}':", cluster.name);
        write!(
            output,
            "{}",
            &toml::to_string_pretty(&launchers.by_cluster(&cluster.name))?
        )?;
    }

    Ok(())
}
