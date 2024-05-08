pub mod cluster;
pub mod directories;
pub mod launchers;
pub mod scan;
pub mod status;
pub mod submit;

use clap::{Args, Parser, Subcommand, ValueEnum};
use clap_verbosity_flag::{Verbosity, WarnLevel};
use log::trace;
use std::io;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None, subcommand_required = true)]
pub struct Options {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[command(flatten)]
    pub global: GlobalOptions,

    #[command(flatten)]
    pub verbose: Verbosity<WarnLevel>,
}

#[derive(Args, Debug, Clone)]
pub struct GlobalOptions {
    /// Number of threads to use for IO intensive operations.
    #[arg(long, default_value_t=8 as u16, global=true, env="ROW_IO_THREADS", display_order=2)]
    pub io_threads: u16,

    /// When to print colored output.
    #[arg(long, value_name="WHEN", value_enum, default_value_t=ColorMode::Auto, global=true, env="ROW_COLOR", display_order=2)]
    pub color: ColorMode,

    /// Disable progress bars.
    #[arg(long, global = true, env = "ROW_NO_PROGRESS", display_order = 2)]
    pub no_progress: bool,

    /// Clear progress bars on exit.
    #[arg(long, global = true, env = "ROW_CLEAR_PROGRESS", display_order = 2)]
    pub clear_progress: bool,

    /// Check the job submission status on the given cluster. Autodetected by default.
    #[arg(long, global = true, env = "ROW_CLUSTER", display_order = 2)]
    cluster: Option<String>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum ColorMode {
    /// Automatically detect when to print colored output.
    Auto,

    /// Always print colored output.
    Always,

    /// Never print colored output.
    Never,
}

#[derive(Subcommand, Debug)]
pub enum ShowCommands {
    /// Show the current state of the workflow.
    Status(status::Arguments),

    /// List directories in the workspace.
    Directories(directories::Arguments),

    /// Show the cluster configuration.
    Cluster(cluster::Arguments),

    /// Show launcher configurations.
    Launchers(launchers::Arguments),
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Show properties of the workspace.
    #[command(subcommand)]
    Show(ShowCommands),

    /// Scan the workspace for completed actions.
    Scan(scan::Arguments),

    /// Submit workflow actions to the scheduler.
    Submit(submit::Arguments),
}

/// Parse directories passed in on the command line.
///
/// # Returns
/// `Ok(Vec<PathBuf>)` listing all the selected directories.
/// - No input selects all project directories.
/// - One "-" input reads directories from stdin.
/// - Otherwise, pass through the given directories from the command line.
///
/// `Err(row::Error)` when there is an error reading from stdin.
///
pub fn parse_directories<F>(
    mut query_directories: Vec<PathBuf>,
    get_all_directories: F,
) -> Result<Vec<PathBuf>, row::Error>
where
    F: FnOnce() -> Result<Vec<PathBuf>, row::Error>,
{
    if query_directories.len() == 1 && query_directories[0] == PathBuf::from("-") {
        trace!("Reading directories from stdin.");
        query_directories.clear();
        for line in io::stdin().lines() {
            query_directories.push(PathBuf::from(line?));
        }
    } else if query_directories.is_empty() {
        trace!("Checking all directories.");
        query_directories = get_all_directories()?;
    }

    Ok(query_directories)
}
