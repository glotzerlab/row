use indicatif::{ProgressBar, ProgressDrawTarget};
use log::{debug, trace, warn};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io;
use std::io::prelude::*;
use std::path::PathBuf;

use crate::workflow::Workflow;
use crate::{
    progress_styles, workspace, Error, MultiProgressContainer, COMPLETED_CACHE_FILE_NAME,
    COMPLETED_DIRECTORY_NAME, DATA_DIRECTORY_NAME, MIN_PROGRESS_BAR_SIZE,
    SUBMITTED_CACHE_FILE_NAME, VALUE_CACHE_FILE_NAME,
};

type SubmittedJobs = HashMap<String, HashMap<PathBuf, (String, u32)>>;

/// The state of the project.
///
/// `State` collects the following information on the workspace and manages cache files
/// on the filesystem for these (separately):
/// * JSON values for each directory
/// * Completed directories for each action.
/// * Scheduled jobs by action, directory, (and cluster?).
///
/// `State` implements methods that synchronize a state with the workspace on disk and
/// to interface with the scheduler's queue.
///
#[derive(Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct State {
    /// The cached value of each directory.
    values: HashMap<PathBuf, Value>,

    /// Completed directories for each action.
    completed: HashMap<String, HashSet<PathBuf>>,

    /// Submitted jobs: action -> directory -> (cluster, job ID)
    submitted: SubmittedJobs,

    /// Completion files read while synchronizing.
    completed_file_names: Vec<PathBuf>,

    /// Set to true when `values` is modified from the on-disk cache.
    values_modified: bool,

    /// Set to true when `completed` is modified from the on-disk cache.
    completed_modified: bool,

    /// Set to true when `submitted` is modified from the on-disk cache.
    submitted_modified: bool,
}

impl State {
    /// Get the directory values.
    pub fn values(&self) -> &HashMap<PathBuf, Value> {
        &self.values
    }

    /// Get the set of directories completed for a given action.
    pub fn completed(&self) -> &HashMap<String, HashSet<PathBuf>> {
        &self.completed
    }

    /// Get the mapping of actions -> directories -> (cluster, submitted job ID)
    pub fn submitted(&self) -> &SubmittedJobs {
        &self.submitted
    }

    /// Test whether a given directory has a submitted job for the given action.
    pub fn is_submitted(&self, action_name: &str, directory: &PathBuf) -> bool {
        if let Some(submitted_directories) = self.submitted.get(action_name) {
            submitted_directories.contains_key(directory)
        } else {
            false
        }
    }

    /// Add a submitted job.
    pub fn add_submitted(
        &mut self,
        action_name: &str,
        directories: &[PathBuf],
        cluster_name: &str,
        job_id: u32,
    ) {
        for directory in directories {
            self.submitted
                .entry(action_name.into())
                .and_modify(|e| {
                    e.insert(directory.clone(), (cluster_name.to_string(), job_id));
                })
                .or_insert(HashMap::from([(
                    directory.clone(),
                    (cluster_name.to_string(), job_id),
                )]));
        }
        self.submitted_modified = true;
    }

    /// Remove inactive jobs on the given cluster.
    ///
    /// Note: The argument lists the *active* jobs to keep!
    ///
    pub fn remove_inactive_submitted(&mut self, cluster_name: &str, active_job_ids: &HashSet<u32>) {
        trace!("Removing inactive jobs from the submitted cache.");
        self.submitted_modified = true;

        for directories in self.submitted.values_mut() {
            directories.retain(|_, v| v.0 != cluster_name || active_job_ids.contains(&v.1));
        }
    }

    /// Get all submitted jobs on a given cluster.
    pub fn jobs_submitted_on(&self, cluster_name: &str) -> Vec<u32> {
        let mut set: HashSet<u32> = HashSet::new();

        for directories in self.submitted.values() {
            for (job_cluster, job_id) in directories.values() {
                if job_cluster == cluster_name {
                    set.insert(*job_id);
                }
            }
        }

        Vec::from_iter(set.drain())
    }

    /// List all directories in the state.
    pub fn list_directories(&self) -> Vec<PathBuf> {
        trace!("Listing all directories in project.");
        let mut result = Vec::with_capacity(self.values.len());
        result.extend(self.values.keys().cloned());
        result
    }

    /// Read the state cache from disk.
    pub fn from_cache(workflow: &Workflow) -> Result<State, Error> {
        let mut state = State {
            values: Self::read_value_cache(workflow)?,
            completed: Self::read_completed_cache(workflow)?,
            submitted: Self::read_submitted_cache(workflow)?,
            completed_file_names: Vec::new(),
            values_modified: false,
            completed_modified: false,
            submitted_modified: false,
        };

        // Ensure that completed has keys for all actions in the workflow.
        for action in &workflow.action {
            if !state.completed.contains_key(&action.name) {
                state.completed.insert(action.name.clone(), HashSet::new());
            }
        }

        Ok(state)
    }

    /// Read the value cache from disk.
    fn read_value_cache(workflow: &Workflow) -> Result<HashMap<PathBuf, Value>, Error> {
        let data_directory = workflow.root.join(DATA_DIRECTORY_NAME);
        let value_file = data_directory.join(VALUE_CACHE_FILE_NAME);

        match fs::read(&value_file) {
            Ok(bytes) => {
                debug!("Reading cache '{}'.", value_file.display().to_string());

                let result =
                    serde_json::from_slice(&bytes).map_err(|e| Error::JSONParse(value_file, e))?;

                Ok(result)
            }
            Err(error) => match error.kind() {
                io::ErrorKind::NotFound => {
                    trace!(
                        "'{}' not found, initializing default values.",
                        value_file.display().to_string()
                    );
                    Ok(HashMap::new())
                }

                _ => Err(Error::FileRead(value_file, error)),
            },
        }
    }

    /// Read the completed directories cache from disk.
    fn read_completed_cache(
        workflow: &Workflow,
    ) -> Result<HashMap<String, HashSet<PathBuf>>, Error> {
        let data_directory = workflow.root.join(DATA_DIRECTORY_NAME);
        let completed_file = data_directory.join(COMPLETED_CACHE_FILE_NAME);

        match fs::read(&completed_file) {
            Ok(bytes) => {
                debug!("Reading cache '{}'.", completed_file.display().to_string());

                let result = postcard::from_bytes(&bytes)
                    .map_err(|e| Error::PostcardParse(completed_file, e))?;
                Ok(result)
            }
            Err(error) => match error.kind() {
                io::ErrorKind::NotFound => {
                    trace!(
                        "'{}' not found, initializing empty completions.",
                        completed_file.display().to_string()
                    );
                    Ok(HashMap::new())
                }

                _ => Err(Error::FileRead(completed_file, error)),
            },
        }
    }

    /// Read the submitted job cache from disk.
    fn read_submitted_cache(workflow: &Workflow) -> Result<SubmittedJobs, Error> {
        let data_directory = workflow.root.join(DATA_DIRECTORY_NAME);
        let submitted_file = data_directory.join(SUBMITTED_CACHE_FILE_NAME);

        match fs::read(&submitted_file) {
            Ok(bytes) => {
                debug!("Reading cache '{}'.", submitted_file.display().to_string());

                let result = postcard::from_bytes(&bytes)
                    .map_err(|e| Error::PostcardParse(submitted_file, e))?;
                Ok(result)
            }
            Err(error) => match error.kind() {
                io::ErrorKind::NotFound => {
                    debug!(
                        "'{}' not found, assuming no submitted jobs.",
                        submitted_file.display().to_string()
                    );
                    Ok(HashMap::new())
                }

                _ => Err(Error::FileRead(submitted_file, error)),
            },
        }
    }

    /// Save the state cache to the filesystem.
    pub fn save_cache(
        &mut self,
        workflow: &Workflow,
        multi_progress: &mut MultiProgressContainer,
    ) -> Result<(), Error> {
        if self.values_modified {
            self.save_value_cache(workflow)?;
            self.values_modified = false;
        }

        if self.completed_modified {
            self.save_completed_cache(workflow, multi_progress)?;
            self.completed_modified = false;
        }

        if self.submitted_modified {
            self.save_submitted_cache(workflow)?;
            self.submitted_modified = false;
        }

        Ok(())
    }

    /// Save the value cache to the filesystem.
    fn save_value_cache(&self, workflow: &Workflow) -> Result<(), Error> {
        let data_directory = workflow.root.join(DATA_DIRECTORY_NAME);
        let value_file = data_directory.join(VALUE_CACHE_FILE_NAME);

        debug!(
            "Saving value cache: '{}'.",
            value_file.display().to_string()
        );

        let out_bytes: Vec<u8> = serde_json::to_vec(&self.values)
            .map_err(|e| Error::JSONSerialize(value_file.clone(), e))?;

        fs::create_dir_all(&data_directory)
            .map_err(|e| Error::DirectoryCreate(data_directory, e))?;
        fs::write(&value_file, out_bytes).map_err(|e| Error::FileWrite(value_file.clone(), e))?;

        Ok(())
    }

    /// Save the completed cache to the filesystem.
    fn save_completed_cache(
        &mut self,
        workflow: &Workflow,
        multi_progress: &mut MultiProgressContainer,
    ) -> Result<(), Error> {
        let data_directory = workflow.root.join(DATA_DIRECTORY_NAME);
        let completed_file = data_directory.join(COMPLETED_CACHE_FILE_NAME);

        debug!(
            "Saving completed cache: '{}'.",
            completed_file.display().to_string()
        );

        // Save the combined cache first.
        let out_bytes: Vec<u8> = postcard::to_stdvec(&self.completed)
            .map_err(|e| Error::PostcardSerialize(completed_file.clone(), e))?;

        let mut file = File::create(&completed_file)
            .map_err(|e| Error::FileWrite(completed_file.clone(), e))?;
        file.write_all(&out_bytes)
            .map_err(|e| Error::FileWrite(completed_file.clone(), e))?;
        file.sync_all()
            .map_err(|e| Error::FileWrite(completed_file.clone(), e))?;
        drop(file);

        // Then remove the staged files.
        let mut progress = ProgressBar::new(self.completed_file_names.len() as u64)
            .with_message("Removing staged completed actions");
        if self.completed_file_names.len() >= MIN_PROGRESS_BAR_SIZE {
            progress = multi_progress.multi_progress.add(progress);
            multi_progress.progress_bars.push(progress.clone());
        } else {
            progress.set_draw_target(ProgressDrawTarget::hidden());
        }
        progress.set_style(progress_styles::counted_bar());
        progress.tick();

        for completed_file_name in &self.completed_file_names {
            trace!("Removing '{}'.", completed_file_name.display().to_string());
            fs::remove_file(completed_file_name)
                .map_err(|e| Error::FileRemove(completed_file_name.clone(), e))?;
        }
        self.completed_file_names.clear();

        progress.finish();
        Ok(())
    }

    /// Save the completed cache to the filesystem.
    fn save_submitted_cache(&mut self, workflow: &Workflow) -> Result<(), Error> {
        let data_directory = workflow.root.join(DATA_DIRECTORY_NAME);
        let submitted_file = data_directory.join(SUBMITTED_CACHE_FILE_NAME);

        debug!(
            "Saving submitted job cache: '{}'.",
            submitted_file.display().to_string()
        );

        let out_bytes: Vec<u8> = postcard::to_stdvec(&self.submitted)
            .map_err(|e| Error::PostcardSerialize(submitted_file.clone(), e))?;

        let mut file = File::create(&submitted_file)
            .map_err(|e| Error::FileWrite(submitted_file.clone(), e))?;
        file.write_all(&out_bytes)
            .map_err(|e| Error::FileWrite(submitted_file.clone(), e))?;
        file.sync_all()
            .map_err(|e| Error::FileWrite(submitted_file.clone(), e))?;
        drop(file);

        Ok(())
    }

    /// Synchronize a workspace on disk with a `State`.
    ///
    /// * Remove directories from the state that are no longer present on the filesystem.
    /// * Make no changes to directories in the state that remain.
    /// * When new directories are present on the filesystem, add them to the state -
    ///   which includes reading the value file and checking which actions are completed.
    /// * Remove actions that are no longer present from the completed and submitted caches.
    /// * Remove directories that are no longer present from the completed and submitted caches.
    ///
    /// # Errors
    ///
    /// * Returns `Error<row::Error>` when there is an I/O error reading the
    ///   workspace directory
    ///
    pub(crate) fn synchronize_workspace(
        &mut self,
        workflow: &Workflow,
        io_threads: u16,
        multi_progress: &mut MultiProgressContainer,
    ) -> Result<&Self, Error> {
        let workspace_path = workflow.root.join(&workflow.workspace.path);

        debug!("Synchronizing workspace '{}'.", workspace_path.display());

        // TODO: get workspace metadata. Store mtime in the cache. Then call `list_directories`
        // only when the current mtime is different from the value in the cache.
        let filesystem_directories: HashSet<PathBuf> =
            HashSet::from_iter(workspace::list_directories(workflow, multi_progress)?);

        ////////////////////////////////////////////////
        // First, synchronize the values.
        // Make a copy of the directories to remove.
        let directories_to_remove: Vec<PathBuf> = self
            .values
            .keys()
            .filter(|&x| !filesystem_directories.contains(x))
            .cloned()
            .collect();

        if directories_to_remove.is_empty() {
            trace!("No directories to remove from the value cache.");
        } else {
            self.values_modified = true;
        }

        // Then remove them.
        for directory in directories_to_remove {
            trace!("Removing '{}' from the value cache", directory.display());
            self.values.remove(&directory);
        }

        // Make a copy of the directories to be added.
        let directories_to_add: Vec<PathBuf> = filesystem_directories
            .iter()
            .filter(|&x| !self.values.contains_key(x))
            .cloned()
            .collect();

        if directories_to_add.is_empty() {
            trace!("No directories to add to the value cache.");
        } else {
            trace!(
                "Adding {} directories to the workspace.",
                directories_to_add.len()
            );
            self.values_modified = true;
        }

        // Read value files from the directories.
        let directory_values = workspace::read_values(
            workflow,
            directories_to_add.clone(),
            io_threads,
            multi_progress,
        );

        ///////////////////////////////////////////
        // Synchronize completed with the disk.

        // Determine which of the new actions are completed.
        let new_complete = workspace::find_completed_directories(
            workflow,
            directories_to_add,
            io_threads,
            multi_progress,
        );

        self.synchronize_completion_files(workflow, multi_progress)?;

        ///////////////////////////////////////////
        // Wait for launched threads to finish and merge results.
        self.values.extend(directory_values.get()?);

        let new_complete = new_complete.get()?;
        if !new_complete.is_empty() {
            self.completed_modified = true;
        }

        self.insert_staged_completed(new_complete);
        self.remove_missing_completed(workflow);
        self.remove_missing_submitted(workflow);

        Ok(self)
    }

    /// Insert new completions.
    fn insert_staged_completed(&mut self, new_complete: HashMap<String, HashSet<PathBuf>>) {
        for (action_name, new_completed_directories) in new_complete {
            if let Some(completed_directories) = self.completed.get_mut(&action_name) {
                completed_directories.extend(new_completed_directories);
            } else {
                self.completed
                    .insert(action_name, new_completed_directories);
            }
        }
    }

    /// Remove missing completed actions and directories.
    fn remove_missing_completed(&mut self, workflow: &Workflow) {
        let current_actions: HashSet<String> =
            workflow.action.iter().map(|a| a.name.clone()).collect();

        let actions_to_remove: Vec<String> = self
            .completed
            .keys()
            .filter(|a| !current_actions.contains(*a))
            .cloned()
            .collect();

        for action_name in actions_to_remove {
            warn!("Removing action '{}' from the completed cache as it is no longer present in the workflow.", action_name);
            self.completed.remove(&action_name);
            self.completed_modified = true;
        }

        for (_, directories) in self.completed.iter_mut() {
            let directories_to_remove: Vec<PathBuf> = directories
                .iter()
                .filter(|d| !self.values.contains_key(*d))
                .cloned()
                .collect();

            for directory_name in directories_to_remove {
                trace!("Removing directory '{}' from the completed cache as it is no longer present in the workspace.", directory_name.display());
                directories.remove(&directory_name);
                self.completed_modified = true;
            }
        }
    }

    /// Remove missing submitted actions and directories.
    fn remove_missing_submitted(&mut self, workflow: &Workflow) {
        let current_actions: HashSet<String> =
            workflow.action.iter().map(|a| a.name.clone()).collect();

        let actions_to_remove: Vec<String> = self
            .submitted
            .keys()
            .filter(|a| !current_actions.contains(*a))
            .cloned()
            .collect();

        for action_name in actions_to_remove {
            warn!("Removing action '{}' from the submitted cache as it is no longer present in the workflow.", action_name);
            self.submitted.remove(&action_name);
            self.submitted_modified = true;
        }

        for (_, directory_map) in self.submitted.iter_mut() {
            let directories_to_remove: Vec<PathBuf> = directory_map
                .keys()
                .filter(|d| !self.values.contains_key(*d))
                .cloned()
                .collect();

            for directory_name in directories_to_remove {
                trace!("Removing directory '{}' from the submitted cache as it is no longer present in the workspace.", directory_name.display());
                directory_map.remove(&directory_name);
                self.submitted_modified = true;
            }
        }

        // Note: A separate method takes care of removing submitted job IDs that are
        // no longer submitted.
    }

    /// Synchronize with completion files on the filesystem.
    fn synchronize_completion_files(
        &mut self,
        workflow: &Workflow,
        multi_progress: &mut MultiProgressContainer,
    ) -> Result<(), Error> {
        let completed_path = workflow
            .root
            .join(DATA_DIRECTORY_NAME)
            .join(COMPLETED_DIRECTORY_NAME);
        debug!(
            "Reading completed files in '{}'.",
            completed_path.display().to_string()
        );

        match completed_path.read_dir() {
            Ok(dirs) => {
                for entry in dirs {
                    let entry =
                        entry.map_err(|e| Error::DirectoryRead(completed_path.clone(), e))?;
                    let path = entry.path();

                    if let Some(extension) = path.extension() {
                        if extension == "postcard" {
                            trace!("Reading '{}'", path.display().to_string());
                            self.completed_file_names.push(path);
                        } else {
                            trace!(
                                "Ignoring non-postcard file '{}'",
                                path.display().to_string()
                            );
                        }
                    }
                }
            }

            Err(error) => match error.kind() {
                io::ErrorKind::NotFound => {
                    trace!("'{}' not found.", completed_path.display().to_string());
                    return Ok(());
                }

                _ => return Err(Error::DirectoryRead(completed_path, error)),
            },
        };

        if self.completed_file_names.is_empty() {
            return Ok(());
        }

        self.completed_modified = true;

        let mut progress = ProgressBar::new(self.completed_file_names.len() as u64)
            .with_message("Reading staged completed actions");
        if self.completed_file_names.len() >= MIN_PROGRESS_BAR_SIZE {
            progress = multi_progress.multi_progress.add(progress);
            multi_progress.progress_bars.push(progress.clone());
        } else {
            progress.set_draw_target(ProgressDrawTarget::hidden());
        }
        progress.set_style(progress_styles::counted_bar());
        progress.tick();

        for completed_file_name in &self.completed_file_names {
            trace!("Reading '{}'.", completed_file_name.display().to_string());
            let bytes = fs::read(completed_file_name)
                .map_err(|e| Error::FileRead(completed_file_name.clone(), e))?;
            let new_complete: HashMap<String, HashSet<PathBuf>> = postcard::from_bytes(&bytes)
                .map_err(|e| Error::PostcardParse(completed_file_name.clone(), e))?;

            for (action_name, new_completed_directories) in new_complete {
                if let Some(completed_directories) = self.completed.get_mut(&action_name) {
                    completed_directories.extend(new_completed_directories);
                } else {
                    self.completed
                        .insert(action_name, new_completed_directories);
                }
            }

            progress.inc(1);
        }

        progress.finish();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use assert_fs::prelude::*;
    use assert_fs::TempDir;
    use indicatif::{MultiProgress, ProgressDrawTarget};
    use serial_test::parallel;

    use super::*;

    fn setup() -> MultiProgressContainer {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::max())
            .is_test(true)
            .try_init();

        let multi_progress = MultiProgress::with_draw_target(ProgressDrawTarget::hidden());
        MultiProgressContainer {
            progress_bars: Vec::new(),
            multi_progress,
        }
    }

    #[test]
    #[parallel]
    fn no_workspace() {
        let mut multi_progress = setup();

        let temp = TempDir::new().unwrap();
        let workflow = "";
        let workflow = Workflow::open_str(temp.path(), workflow).unwrap();

        let mut state = State::default();
        let result = state.synchronize_workspace(&workflow, 2, &mut multi_progress);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .starts_with("Unable to read"));
    }

    #[test]
    #[parallel]
    fn empty_workspace() {
        let mut multi_progress = setup();

        let temp = TempDir::new().unwrap();
        temp.child("workspace").create_dir_all().unwrap();
        let workflow = "";
        let workflow = Workflow::open_str(temp.path(), workflow).unwrap();

        let mut state = State::default();
        let result = state.synchronize_workspace(&workflow, 2, &mut multi_progress);
        assert!(result.is_ok());
        assert_eq!(state.values.len(), 0);
    }

    #[test]
    #[parallel]
    fn add_remove() {
        let mut multi_progress = setup();

        let temp = TempDir::new().unwrap();
        temp.child("workspace")
            .child("dir1")
            .create_dir_all()
            .unwrap();
        temp.child("workspace")
            .child("dir2")
            .create_dir_all()
            .unwrap();
        temp.child("workspace")
            .child("dir3")
            .create_dir_all()
            .unwrap();
        let workflow = "";
        let workflow = Workflow::open_str(temp.path(), workflow).unwrap();

        let mut state = State::default();
        state.values.insert(PathBuf::from("dir4"), Value::Null);

        let result = state.synchronize_workspace(&workflow, 2, &mut multi_progress);
        assert!(result.is_ok());

        assert_eq!(state.values.len(), 3);
        assert!(state.values.contains_key(&PathBuf::from("dir1")));
        assert!(state.values.contains_key(&PathBuf::from("dir2")));
        assert!(state.values.contains_key(&PathBuf::from("dir3")));
    }

    #[test]
    #[parallel]
    fn value() {
        let mut multi_progress = setup();

        let temp = TempDir::new().unwrap();
        let dir1 = temp.child("workspace").child("dir1");
        dir1.create_dir_all().unwrap();

        dir1.child("v.json")
            .write_str(&serde_json::to_value(10).unwrap().to_string())
            .unwrap();

        let workflow = r#"workspace.value_file = "v.json""#;
        let workflow = Workflow::open_str(temp.path(), workflow).unwrap();

        let mut state = State::default();

        let result = state.synchronize_workspace(&workflow, 2, &mut multi_progress);
        assert!(result.is_ok());
        assert_eq!(state.values.len(), 1);
        assert!(state.values.contains_key(&PathBuf::from("dir1")));
        assert_eq!(state.values[&PathBuf::from("dir1")].as_i64(), Some(10));
    }

    fn setup_completion_directories(temp: &TempDir, n: usize) -> String {
        for i in 0..n {
            let directory = temp.child("workspace").child(format!("dir{i}"));
            directory.create_dir_all().unwrap();
            directory.child("v").write_str(&format!("{i}")).unwrap();

            if i < n / 2 {
                directory.child("d").touch().unwrap();
            } else {
                directory.child("g").touch().unwrap();
            }
        }

        r#"
[workspace]
value_file = "v"

[[action]]
name = "b"
command = "c"
products = ["d"]

[[action]]
name = "e"
command = "f"
products = ["g"]
"#
        .to_string()
    }

    #[test]
    #[parallel]
    fn new_completeions_and_cache() {
        let mut multi_progress = setup();

        let temp = TempDir::new().unwrap();
        let n = 10;

        let workflow = setup_completion_directories(&temp, n);
        let workflow = Workflow::open_str(temp.path(), &workflow).unwrap();

        let mut state = State::default();
        let result = state.synchronize_workspace(&workflow, 2, &mut multi_progress);
        assert!(result.is_ok());

        assert_eq!(state.values.len(), n);
        assert!(state.completed.contains_key("b"));
        assert!(state.completed.contains_key("e"));
        for i in 0..n {
            let directory = PathBuf::from(format!("dir{i}"));
            assert_eq!(state.values[&directory].as_i64().unwrap() as usize, i);

            if i < n / 2 {
                assert!(state.completed["b"].contains(&directory));
                assert!(!state.completed["e"].contains(&directory));
            } else {
                assert!(!state.completed["b"].contains(&directory));
                assert!(state.completed["e"].contains(&directory));
            }
        }

        state
            .save_cache(&workflow, &mut multi_progress)
            .expect("Cache saved.");

        let cached_state = State::from_cache(&workflow).expect("Read state from cache");
        assert_eq!(state, cached_state);
    }

    #[test]
    #[parallel]
    fn completions_not_synced_for_known_directories() {
        let mut multi_progress = setup();

        let temp = TempDir::new().unwrap();
        let n = 10;

        let mut state = State::default();
        for i in 0..n {
            state
                .values
                .insert(PathBuf::from(format!("dir{i}")), Value::Null);
        }

        let workflow = setup_completion_directories(&temp, n);
        let workflow = Workflow::open_str(temp.path(), &workflow).unwrap();
        let result = state.synchronize_workspace(&workflow, 2, &mut multi_progress);
        assert!(result.is_ok());

        assert_eq!(state.values.len(), n);
        assert!(!state.completed.contains_key("b"));
        assert!(!state.completed.contains_key("e"));
    }

    #[test]
    #[parallel]
    fn completed_removed() {
        let mut multi_progress = setup();

        let temp = TempDir::new().unwrap();
        let n = 10;

        let workflow = setup_completion_directories(&temp, n);

        let mut state = State::default();
        state.completed.insert(
            "b".to_string(),
            HashSet::from([PathBuf::from("notdir100"), PathBuf::from("notdir200")]),
        );
        state.completed.insert(
            "e".to_string(),
            HashSet::from([PathBuf::from("notdir50"), PathBuf::from("notdir80")]),
        );
        state.completed.insert(
            "z".to_string(),
            HashSet::from([PathBuf::from("dir1"), PathBuf::from("dir2")]),
        );

        let workflow = Workflow::open_str(temp.path(), &workflow).unwrap();
        let result = state.synchronize_workspace(&workflow, 2, &mut multi_progress);
        assert!(result.is_ok());

        assert_eq!(state.values.len(), n);
        assert!(state.completed.contains_key("b"));
        assert!(state.completed.contains_key("e"));
        assert!(!state.completed.contains_key("z"));

        assert_eq!(state.completed["b"].len(), n / 2);
        assert_eq!(state.completed["e"].len(), n / 2);

        for i in 0..n {
            let directory = PathBuf::from(format!("dir{i}"));
            assert_eq!(state.values[&directory].as_i64().unwrap() as usize, i);

            if i < n / 2 {
                assert!(state.completed["b"].contains(&directory));
                assert!(!state.completed["e"].contains(&directory));
            } else {
                assert!(!state.completed["b"].contains(&directory));
                assert!(state.completed["e"].contains(&directory));
            }
        }
    }

    #[test]
    #[parallel]
    fn new_submitted_and_cache() {
        let mut multi_progress = setup();

        let temp = TempDir::new().unwrap();
        let n = 8;

        let workflow = setup_completion_directories(&temp, n);
        let workflow = Workflow::open_str(temp.path(), &workflow).unwrap();

        let mut state = State::default();
        let result = state.synchronize_workspace(&workflow, 2, &mut multi_progress);
        assert!(result.is_ok());

        assert!(state.submitted.is_empty());

        state.add_submitted("b", &["dir1".into(), "dir5".into()], "cluster1", 11);
        state.add_submitted("b", &["dir3".into(), "dir4".into()], "cluster2", 12);
        state.add_submitted("e", &["dir6".into(), "dir7".into()], "cluster2", 13);

        assert!(state.is_submitted("b", &"dir1".into()));
        assert!(!state.is_submitted("b", &"dir2".into()));
        assert!(state.is_submitted("b", &"dir3".into()));
        assert!(state.is_submitted("b", &"dir4".into()));
        assert!(state.is_submitted("b", &"dir5".into()));
        assert!(!state.is_submitted("b", &"dir6".into()));
        assert!(!state.is_submitted("b", &"dir7".into()));

        assert!(!state.is_submitted("e", &"dir1".into()));
        assert!(!state.is_submitted("e", &"dir2".into()));
        assert!(!state.is_submitted("e", &"dir3".into()));
        assert!(!state.is_submitted("e", &"dir4".into()));
        assert!(!state.is_submitted("e", &"dir5".into()));
        assert!(state.is_submitted("e", &"dir6".into()));
        assert!(state.is_submitted("e", &"dir7".into()));

        assert_eq!(state.jobs_submitted_on("cluster1"), vec![11]);
        let mut jobs_on_cluster2 = state.jobs_submitted_on("cluster2");
        jobs_on_cluster2.sort();
        assert_eq!(jobs_on_cluster2, vec![12, 13]);

        state
            .save_cache(&workflow, &mut multi_progress)
            .expect("Cache saved.");

        let cached_state = State::from_cache(&workflow).expect("Read state from cache");
        assert_eq!(state, cached_state);
    }

    #[test]
    #[parallel]
    fn remove_submitted_actions_and_dirs() {
        let mut multi_progress = setup();

        let temp = TempDir::new().unwrap();
        let n = 8;

        let workflow = setup_completion_directories(&temp, n);
        let workflow = Workflow::open_str(temp.path(), &workflow).unwrap();

        let mut state = State::default();
        let result = state.synchronize_workspace(&workflow, 2, &mut multi_progress);
        assert!(result.is_ok());

        assert!(state.submitted.is_empty());

        state.add_submitted("b", &["dir25".into(), "dir27".into()], "cluster1", 18);
        state.add_submitted("b", &["dir1".into(), "dir2".into()], "cluster1", 19);
        state.add_submitted("f", &["dir3".into(), "dir4".into()], "cluster2", 27);

        assert!(state.is_submitted("b", &"dir1".into()));
        assert!(state.is_submitted("b", &"dir2".into()));
        assert!(state.is_submitted("b", &"dir25".into()));
        assert!(state.is_submitted("b", &"dir27".into()));

        assert!(state.is_submitted("f", &"dir3".into()));
        assert!(state.is_submitted("f", &"dir4".into()));

        state
            .save_cache(&workflow, &mut multi_progress)
            .expect("Cache saved.");

        let mut cached_state = State::from_cache(&workflow).expect("Read state from cache");
        assert_eq!(state, cached_state);

        let result = cached_state.synchronize_workspace(&workflow, 2, &mut multi_progress);
        assert!(result.is_ok());

        assert!(!cached_state.submitted.contains_key("f"));
        assert!(!cached_state.is_submitted("f", &"dir3".into()));
        assert!(!cached_state.is_submitted("f", &"dir4".into()));

        assert!(cached_state.is_submitted("b", &"dir1".into()));
        assert!(cached_state.is_submitted("b", &"dir2".into()));
        assert!(!cached_state.is_submitted("b", &"dir25".into()));
        assert!(!cached_state.is_submitted("b", &"dir27".into()));
    }

    #[test]
    #[parallel]
    fn remove_inactive() {
        let mut multi_progress = setup();

        let temp = TempDir::new().unwrap();
        let n = 8;

        let workflow = setup_completion_directories(&temp, n);
        let workflow = Workflow::open_str(temp.path(), &workflow).unwrap();

        let mut state = State::default();
        let result = state.synchronize_workspace(&workflow, 2, &mut multi_progress);
        assert!(result.is_ok());

        assert!(state.submitted.is_empty());

        state.add_submitted("b", &["dir1".into(), "dir5".into()], "cluster1", 11);
        state.add_submitted("b", &["dir3".into(), "dir4".into()], "cluster2", 12);
        state.add_submitted("e", &["dir6".into(), "dir7".into()], "cluster2", 13);

        assert!(state.is_submitted("b", &"dir1".into()));
        assert!(!state.is_submitted("b", &"dir2".into()));
        assert!(state.is_submitted("b", &"dir3".into()));
        assert!(state.is_submitted("b", &"dir4".into()));
        assert!(state.is_submitted("b", &"dir5".into()));
        assert!(!state.is_submitted("b", &"dir6".into()));
        assert!(!state.is_submitted("b", &"dir7".into()));

        assert!(!state.is_submitted("e", &"dir1".into()));
        assert!(!state.is_submitted("e", &"dir2".into()));
        assert!(!state.is_submitted("e", &"dir3".into()));
        assert!(!state.is_submitted("e", &"dir4".into()));
        assert!(!state.is_submitted("e", &"dir5".into()));
        assert!(state.is_submitted("e", &"dir6".into()));
        assert!(state.is_submitted("e", &"dir7".into()));

        state.remove_inactive_submitted("cluster2", &HashSet::from([13]));
        assert!(state.is_submitted("b", &"dir1".into()));
        assert!(state.is_submitted("b", &"dir5".into()));
        assert!(!state.is_submitted("b", &"dir3".into()));
        assert!(!state.is_submitted("b", &"dir4".into()));
        assert!(state.is_submitted("e", &"dir6".into()));
        assert!(state.is_submitted("e", &"dir7".into()));

        state.remove_inactive_submitted("cluster1", &HashSet::from([]));
        assert!(!state.is_submitted("b", &"dir1".into()));
        assert!(!state.is_submitted("b", &"dir5".into()));
    }
}
