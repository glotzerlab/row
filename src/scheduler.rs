// Copyright (c) 2024 The Regents of the University of Michigan.
// Part of row, released under the BSD 3-Clause License.

pub mod bash;
pub mod slurm;

use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::workflow::Action;
use crate::Error;

/// A `Scheduler` creates and submits job scripts.
pub trait Scheduler {
    /** Make a job script given an `Action` and a list of directories.

    # Arguments
    * `action`: The action to submit.
    * `directories`: The directories to include in the submission.
    * `workspace_path`: The relative path to the workspace directory from the workflow root.
    * `directory_values`: Maps directory names to JSON values.

    `make_script` must use expand `{workspace_path`} and `{\JSON pointer}`
    templates in the action's command.

    # Returns
    A `String` containing the job script.

    # Errors
    Returns `Err<row::Error>` when the script cannot be created.
    */
    fn make_script(
        &self,
        action: &Action,
        directories: &[PathBuf],
        workspace_path: &Path,
        directory_values: &HashMap<PathBuf, Value>,
    ) -> Result<String, Error>;

    /** Submit a job to the scheduler.

    # Arguments
    * `workflow_root`: The working directory the action should be submitted from.
    * `action`: The action to submit.
    * `directories`: The directories to include in the submission.
    * `workspace_path`: The relative path to the workspace directory from the workflow root.
    * `directory_values`: Maps directory names to JSON values.
    * `should_terminate`: Set to true when the user terminates the process.

    # Returns
    `Ok(job_id_option)` on success.
    Schedulers that queue jobs should set `job_id_option = Some(job_id)`.
    Schedulers that execute jobs immediately should set `job_id_option = None`.

    # Early termination.
    Implementations should periodically check `should_terminate` and
    exit early (if possible) with `Err(Error::Interrupted)` when set.

    # Errors
    Returns `Err(row::Error)` on error, which may be due to a non-zero exit
    status from the submission.
    */
    fn submit(
        &self,
        workflow_root: &Path,
        action: &Action,
        directories: &[PathBuf],
        workspace_path: &Path,
        directory_values: &HashMap<PathBuf, Value>,
        should_terminate: Arc<AtomicBool>,
    ) -> Result<Option<u32>, Error>;

    /// Query the scheduler and determine which jobs remain active.
    ///
    /// # Arguments
    /// * `jobs`: Identifiers to query
    ///
    /// `active_jobs` returns a `ActiveJobs` object, which provides the final
    /// result via a method. This allows implementations to be asynchronous so
    /// that long-running subprocesses can complete in the background while the
    /// collar performs other work.
    ///
    /// # Errors
    /// Returns `Err<row::Error>` when the job queue query cannot be executed.
    ///
    fn active_jobs(&self, jobs: &[u32]) -> Result<Box<dyn ActiveJobs>, Error>;
}

/// Deferred result containing jobs that are still active on the cluster.
pub trait ActiveJobs {
    /// Complete the operation and return the currently active jobs.
    ///
    /// # Errors
    /// Returns `Err<row::Error>` when the job queue query cannot be executed.
    ///
    fn get(self: Box<Self>) -> Result<HashSet<u32>, Error>;
}
