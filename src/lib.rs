// Copyright (c) 2024 The Regents of the University of Michigan.
// Part of row, released under the BSD 3-Clause License.

#![warn(clippy::pedantic)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::must_use_candidate)]
#![warn(clippy::format_push_string)]

pub(crate) mod builtin;
pub mod cluster;
mod expr;
pub mod format;
pub mod launcher;
pub mod progress_styles;
pub mod project;
pub mod scheduler;
pub mod state;
pub mod workflow;
pub mod workspace;

use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget};
use serde_json::{self, Value};
use std::io;
use std::path::PathBuf;
use std::sync::mpsc;

pub const DATA_DIRECTORY_NAME: &str = ".row";
pub const COMPLETED_DIRECTORY_NAME: &str = "completed";
pub const MIN_PROGRESS_BAR_SIZE: usize = 1;

pub const DIRECTORY_CACHE_FILE_NAME: &str = "directories.json";
pub const COMPLETED_CACHE_FILE_NAME: &str = "completed.postcard";
pub const SUBMITTED_CACHE_FILE_NAME: &str = "submitted.postcard";

/// Hold a `MultiProgress` and all of its progress bars.
///
/// This is necessary because a dropped `ProgressBar` will be automatically
/// removed from [MultiProgress](https://github.com/console-rs/indicatif/issues/614)
///
pub struct MultiProgressContainer {
    progress_bars: Vec<ProgressBar>,
    multi_progress: MultiProgress,
}

/// Errors that may be encountered when using the row crate.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    // OS errors
    #[error("OS error")]
    OS(#[from] nix::errno::Errno),

    #[error("No home directory")]
    NoHome(),

    // IO errors
    #[error("I/O error: {0}")]
    IO(#[from] io::Error),

    #[error("Unable to find the path to the current executable: {0}")]
    FindCurrentExecutable(#[source] io::Error),

    #[error("Unable to read '{0}': {1}")]
    FileRead(PathBuf, #[source] io::Error),

    #[error("Unable to write '{0}': {1}")]
    FileWrite(PathBuf, #[source] io::Error),

    #[error("Unable to remove '{0}': {1}")]
    FileRemove(PathBuf, #[source] io::Error),

    #[error("File '{0}' already exists.")]
    FileExists(PathBuf),

    #[error("Unable to read '{0}': {1}")]
    DirectoryRead(PathBuf, #[source] io::Error),

    #[error("Directory '{0}' not found in workspace.")]
    DirectoryNotFound(PathBuf),

    #[error("Unable to create directory '{0}': {1}")]
    DirectoryCreate(PathBuf, #[source] io::Error),

    #[error("Non-UTF-8 directory name '{0}'")]
    NonUTF8DirectoryName(PathBuf),

    #[error("Unable to spawn '{0}': {1}.")]
    SpawnProcess(String, #[source] io::Error),

    // serialization errors
    #[error("Unable to parse '{0}'.\n{1}")]
    TOMLParse(PathBuf, #[source] toml::de::Error),

    #[error("Unable to parse '{0}'\n{1}")]
    JSONParse(PathBuf, #[source] serde_json::Error),

    #[error("Unable to serialize '{0}'\n{1}")]
    JSONSerialize(PathBuf, #[source] serde_json::Error),

    #[error("Unable to parse '{0}': {1}")]
    PostcardParse(PathBuf, #[source] postcard::Error),

    #[error("Unable to serialize '{0}': {1}")]
    PostcardSerialize(PathBuf, #[source] postcard::Error),

    // workflow errors
    #[error("Found duplicate action definition '{0}'.")]
    DuplicateAction(String),

    #[error("Previous action '{0}' not found in action '{1}'.")]
    PreviousActionNotFound(String, String),

    #[error("Define 'processes' or 'processes_per_directory', not both in action '{0}'.")]
    DuplicateProcesses(String),

    #[error("Use '{{directory}}' or '{{directories}}', not both in the command of action '{0}'.")]
    ActionContainsMultipleTemplates(String),

    #[error("Use '{{directory}}' or '{{directories}}' in the command of action '{0}'.")]
    ActionContainsNoTemplate(String),

    #[error("workflow.toml not found in the current working directory or any parents.")]
    WorkflowNotFound,

    #[error("The value in directory '{0}' does not contain the JSON pointer '{1}'.")]
    JSONPointerNotFound(PathBuf, String),

    #[error("Cannot compare {0} and {1} while checking directory '{2}'.")]
    CannotCompareInclude(Value, Value, PathBuf),

    // submission errors
    #[error("Error encountered while executing action '{0}': {1}.")]
    ExecuteAction(String, String),

    #[error("Error encountered while submitting action '{0}': {1}.")]
    SubmitAction(String, String),

    #[error("Unepxected output from {0}: {1}")]
    UnexpectedOutput(String, String),

    #[error("Error encountered while running squeue: {0}.\n{1}")]
    ExecuteSqueue(String, String),

    #[error("Interrupted")]
    Interrupted,

    // launcher errors
    #[error("Launcher '{0}' does not contain a default configuration")]
    LauncherMissingDefault(String),

    #[error("Launcher '{0}' not found: Required by action '{1}'.")]
    LauncherNotFound(String, String),

    #[error("No process launcher for action '{0}' which requests {1} processes.")]
    NoProcessLauncher(String, usize),

    #[error("More than one process launcher for action '{0}'.")]
    TooManyProcessLaunchers(String),

    // cluster errors
    #[error(
        "Cluster '{0}' not found: execute 'row show cluster --all' to see available clusters."
    )]
    ClusterNameNotFound(String),

    #[error("No cluster found: execute 'row show cluster -vvv' to see why.")]
    ClusterNotFound(),

    #[error("Partition '{0}' not found: execute 'row show cluster' to see available partitions.")]
    PartitionNameNotFound(String),

    #[error("No valid partitions:\n{0}\nExecute 'row show cluster' to see available partitions.")]
    PartitionNotFound(String),

    // command errors
    #[error("Action '{0}' not found in the workflow.")]
    ActionNotFound(String),

    #[error("A row project already exists in '{0}'.")]
    ProjectExists(PathBuf),

    #[error("A row project already exists in the parent directory '{0}'.")]
    ParentProjectExists(PathBuf),

    #[error("The cache directory '.row' already exists in '{0}'.")]
    ProjectCacheExists(PathBuf),

    #[error("workspace must be a relative path name, got '{0}'.")]
    WorkspacePathNotRelative(String),

    #[error("There are submitted jobs. Rerun with --force to bypass this check.")]
    ForceCleanNeeded,

    #[error("Attempting partial submission of action '{0}' when `submit_whole=true`.")]
    PartialGroupSubmission(String),

    // thread errors
    #[error("Unexpected error communicating between threads in 'find_completed_directories'.")]
    CompletedDirectoriesSend(#[from] mpsc::SendError<(PathBuf, String)>),

    #[error("Unexpected error communicating between threads in 'read_values'.")]
    ReadValuesSend(#[from] mpsc::SendError<(PathBuf, Value)>),
}

impl MultiProgressContainer {
    /// Create a new multi-progress container.
    pub fn new(multi_progress: MultiProgress) -> MultiProgressContainer {
        MultiProgressContainer {
            progress_bars: Vec::new(),
            multi_progress,
        }
    }

    /// Add a progress bar to the container or hide it.
    pub fn add_or_hide(&mut self, mut progress_bar: ProgressBar, hide: bool) -> ProgressBar {
        if hide {
            progress_bar.set_draw_target(ProgressDrawTarget::hidden());
        } else {
            progress_bar = self.multi_progress.add(progress_bar);
            self.progress_bars.push(progress_bar.clone());
        }

        progress_bar
    }

    /// Add a progress bar to the container.
    pub fn add(&mut self, progress_bar: ProgressBar) -> ProgressBar {
        self.progress_bars.push(progress_bar.clone());
        self.multi_progress.add(progress_bar)
    }

    /// Clear all progress bars
    ///
    /// # Errors
    /// Forwards the error from `indicatif::MultiProgress::clear`.
    pub fn clear(&mut self) -> Result<(), std::io::Error> {
        self.progress_bars.clear();
        self.multi_progress.clear()
    }

    /// Suspend the progress bar updates while executing f.
    pub fn suspend<F: FnOnce() -> R, R>(&self, f: F) -> R {
        self.multi_progress.suspend(f)
    }
}
