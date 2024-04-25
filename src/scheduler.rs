pub mod bash;

use std::path::Path;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::workflow::Action;
use crate::Error;

/// A `Scheduler` creates and submits job scripts.
pub trait Scheduler {
    // TODO: give new schedulers the Launcher and partition instances from Cluster.
    fn new(cluster_name: &str) -> Self;

    /// Make a job script given an `Action` and a list of directories.
    ///
    /// Useful for showing the script that would be submitted to the user.
    ///
    /// # Returns
    /// A `String` containing the job script.
    fn make_script(&self, action: &Action, directories: &[PathBuf]) -> Result<String, Error>;

    /// Submit a job to the scheduler.
    ///
    /// # Arguments
    /// * `working_directory`: The working directory the action should be submitted from.
    /// * `action`: The action to submit.
    /// * `directories`: The directories to include in the submission.
    /// * `should_terminate`: Set to true when the user terminates the process.
    ///
    /// # Returns
    /// `Ok(job_id_option)` on success.
    /// `Err(row::Error)` on error, which may be due to a non-zero exit status
    /// from the submission.
    /// Schedulers that queue jobs should set `job_id_option = Some(job_id)`.
    /// Schedulers that execute jobs immediately should set `job_id_option = None`.
    ///
    /// # Early termination.
    /// Implementations should periodically check `should_terminate` and
    /// exit early (if possible) with `Err(Error::Interrupted)` when set.
    ///
    fn submit(
        &self,
        working_directory: &Path,
        action: &Action,
        directories: &[PathBuf],
        should_terminate: Arc<AtomicBool>,
    ) -> Result<Option<u32>, Error>;

    // TODO: status -> run squeue and determine running jobs.
}
