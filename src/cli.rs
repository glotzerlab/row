// Copyright (c) 2024 The Regents of the University of Michigan.
// Part of row, released under the BSD 3-Clause License.

pub mod clean;
pub mod cluster;
pub mod directories;
pub mod init;
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

    /// Check the job submission status on the given cluster.
    ///
    /// Autodetected by default.
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
    ///
    /// `row show status` prints a summary of all actions in the workflow.
    /// The summary includes the number of directories in each status and an
    /// estimate of the remaining cost in either CPU-hours or GPU-hours based
    /// on the number of submitted, eligible, and waiting jobs and the
    /// resources used by the action.
    ///
    /// EXAMPLES
    ///
    /// * Show the status of the entire workspace:
    ///
    ///   row show status
    ///
    /// * Show the status of all actions with eligible directories
    ///
    ///   row show status --eligible
    ///
    /// * Show the status of a specific action:
    ///
    ///   row show status --action=action
    ///
    /// * Show the status of all action names that match a wildcard pattern:
    ///
    ///   row show status --action='project*'
    ///
    /// * Show the status of specific directories in the workspace:
    ///
    ///   row show status directory1 directory2
    ///
    Status(status::Arguments),

    /// List directories in the workspace.
    ///
    /// `row show directories` lists each selected directory with its status
    /// and scheduler job ID (when submitted). for the given `<ACTION`>. You
    /// can also show elements from the directory's value, accessed by JSON
    /// pointer. Blank lines separate groups.
    ///
    /// By default, `row show status` displays directories with any status. Set
    /// one or more of `--completed`, `--submitted`, `--eligible`, and
    /// `--waiting` to show specific directories that have specific statuses.
    ///
    /// EXAMPLES
    ///
    /// * Show all the directories for action `one`:
    ///
    ///   row show directories one
    ///
    /// * Show the directory value element `/value`:
    ///
    ///   row show directories action --value=/value
    ///
    /// * Show specific directories:
    ///
    ///   row show directories action directory1 directory2
    ///
    /// * Show eligible directories
    ///
    ///   row show directories action --eligible
    ///
    Directories(directories::Arguments),

    /// Show the cluster configuration.
    ///
    /// Print the current cluster configuration in TOML format.
    ///
    /// EXAMPLES
    ///
    /// * Show the autodetected cluster:
    ///
    ///   row show cluster
    ///
    /// * Show the configuration of a specific cluster:
    ///
    ///   row show cluster --cluster=anvil
    ///
    /// * Show all clusters:
    ///
    ///   row show cluster --all
    ///
    Cluster(cluster::Arguments),

    /// Show launcher configurations.
    ///
    /// Print the launchers defined for the current cluster (or the cluster
    /// given in `--cluster`). The output is TOML formatted.
    ///
    /// This includes the user-provided launchers in `launchers.toml` and the
    /// built-in launchers (or the user-provided overrides).
    ///
    /// EXAMPLES
    ///
    ///* Show the launchers for the autodetected cluster:
    ///
    ///  row show launchers
    ///
    ///* Show the launchers for a specific cluster:
    ///
    ///  row show launchers --cluster=anvil
    ///
    ///* Show all launchers:
    ///
    ///  row show launchers --all
    ///
    Launchers(launchers::Arguments),
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize a new project.
    ///
    /// `row init` creates `workflow.toml` and the workspace directory in the
    /// given DIRECTORY. It creates the directory if needed. The default workspace
    /// path name is `workspace`. Use the `--workspace` option to change this.
    ///
    /// Set the `--signac` option to create a project compatible with signac.
    /// You must separately initialize the signac project.
    ///
    /// ERRORS
    ///
    /// `row init` returns an error when a row project already exists at the
    /// given DIRECTORY.
    ///
    /// EXAMPLES
    ///
    /// * Create a project in the current directory:
    ///
    /// row init .
    ///
    /// * Create a signac compatible project in the directory `project`:
    ///
    /// row init --signac project
    ///
    /// * Create a project where the workspace is named `data`:
    ///
    /// row init --workspace data project
    ///
    Init(init::Arguments),

    /// Show properties of the workspace.
    #[command(subcommand)]
    Show(ShowCommands),

    /// Scan the workspace for completed actions.
    ///
    /// `row scan` scans the selected directories for action products and
    /// updates the cache of completed directories accordingly.
    ///
    /// EXAMPLES
    ///
    /// * Scan all directories for all actions:
    ///
    ///   row scan
    ///
    /// * Scan a specific action:
    ///
    ///   row scan --action=action
    ///
    /// * Scan specific directories:
    ///
    ///   row scan directory1 directory2
    ///
    Scan(scan::Arguments),

    /// Submit workflow actions to the scheduler.
    ///
    /// `row submit` submits jobs to the scheduler. First it determines the
    /// status of all the given directories for the selected actions. Then it
    /// forms groups and submits one job for each group. Pass `--dry-run` to see
    /// the script(s) that will be submitted.
    ///
    /// EXAMPLES
    ///
    /// * Print the job script(s) that will be submitted:
    ///
    /// row submit --dry-run
    ///
    /// * Submit jobs for all eligible directories:
    ///
    /// row submit
    ///
    /// * Submit the first eligible job:
    ///
    /// row submit -n 1
    ///
    /// * Submit jobs for a specific action:
    ///
    /// row submit --action=action
    ///
    /// * Submit jobs for all actions that match a wildcard pattern:
    ///
    /// row submit --action='project*'
    ///
    /// * Submit jobs on specific directories:
    ///
    /// row submit directory1 directory2
    ///
    Submit(submit::Arguments),

    /// Remove cache files.
    ///
    /// `row clean` safely removes cache files generated by row.
    ///
    /// EXAMPLES
    ///
    /// * Remove the completed cache:
    ///
    ///   row clean --completed
    ///
    Clean(clean::Arguments),
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
