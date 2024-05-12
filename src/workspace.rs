// Copyright (c) 2024 The Regents of the University of Michigan.
// Part of row, released under the BSD 3-Clause License.

use indicatif::ProgressBar;
use log::debug;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::workflow::Workflow;
use crate::{progress_styles, Error, MultiProgressContainer, MIN_PROGRESS_BAR_SIZE};

/// List all directories in the workspace as found on the filesystem.
///
/// # Errors
/// Returns `Err<row::Error>` when the workspace directory cannot be accessed.
///
pub fn list_directories(
    workflow: &Workflow,
    multi_progress: &mut MultiProgressContainer,
) -> Result<Vec<PathBuf>, Error> {
    let workspace_path = workflow.root.join(&workflow.workspace.path);

    let progress = multi_progress.add(ProgressBar::new_spinner().with_message("Listing workspace"));
    progress.set_style(progress_styles::counted_spinner());
    progress.enable_steady_tick(Duration::from_millis(progress_styles::STEADY_TICK));

    let mut directories = Vec::new();

    for entry in workspace_path
        .read_dir()
        .map_err(|e| Error::DirectoryRead(workspace_path.clone(), e))?
    {
        match entry {
            Ok(ref entry) => {
                let file_type = entry
                    .file_type()
                    .map_err(|e| Error::DirectoryRead(workspace_path.clone(), e))?;

                if file_type.is_dir() {
                    progress.inc(1);
                    directories.push(PathBuf::from(entry.file_name()));
                }
            }
            Err(e) => {
                return Err(Error::DirectoryRead(workspace_path, e));
            }
        }
    }

    progress.finish();

    Ok(directories)
}

/// Directories that have completed actions.
///
/// Call `get()` to wait for all pending threads to complete and return the result.
///
pub struct CompletedDirectories {
    /// Threads scanning the directories.
    threads: Vec<JoinHandle<Result<(), Error>>>,

    /// Channel to receive results from worker threads.
    receiver: Receiver<(PathBuf, String)>,

    /// Progress bar.
    progress: ProgressBar,
}

/// Find directories that have completed actions.
///
/// `find_completed_directories` spawns threads to scan the workspace and then
/// returns immediately. Calling `get` on the result will wait for the threads
/// to complete and then provides the list of completions.
///
/// # Arguments
/// * `workflow` - The `Workflow` to scan for completed directories.
/// * `directories` - The directories to scan. Must be present in the workspace.
/// * `io_threads` - Number of threads to use while scanning directories.
///
/// # Panics
/// When unable to spawn threads.
///
pub fn find_completed_directories(
    workflow: &Workflow,
    directories: Vec<PathBuf>,
    io_threads: u16,
    multi_progress: &mut MultiProgressContainer,
) -> CompletedDirectories {
    let mut progress =
        ProgressBar::new(directories.len() as u64).with_message("Scanning directories");
    progress = multi_progress.add_or_hide(progress, directories.len() < MIN_PROGRESS_BAR_SIZE);
    progress.set_style(progress_styles::counted_bar());
    progress.tick();

    if !directories.is_empty() {
        debug!("Finding completed directories.");
    }

    let workspace_path = workflow.root.join(&workflow.workspace.path);
    let directories_mutex = Arc::new(Mutex::new(directories));
    let (sender, receiver) = mpsc::channel();

    let mut action_products: Vec<(String, Vec<String>)> = Vec::new();
    for action in &workflow.action {
        if !action.products.is_empty() {
            action_products.push((action.name.clone(), action.products.clone()));
        }
    }

    let mut threads = Vec::with_capacity(io_threads as usize);

    for i in 0..io_threads {
        let action_products = action_products.clone();
        let workspace_path = workspace_path.clone();
        let directories_mutex = directories_mutex.clone();
        let sender = sender.clone();
        let progress = progress.clone();

        let thread_name = format!("find-completed-{i}");
        let handle =
            thread::Builder::new()
                .name(thread_name)
                .spawn(move || -> Result<(), Error> {
                    let mut directory_path = workspace_path;
                    let mut directory_contents = HashSet::new();

                    loop {
                        let current_directory;

                        // Pull the next directory to process off the shared stack.
                        {
                            let mut directories = directories_mutex.lock().unwrap();
                            if let Some(d) = directories.pop() {
                                current_directory = d;
                            } else {
                                break Ok(());
                            }
                        }

                        // List all files in the current directory.
                        directory_path.push(&current_directory);

                        for entry in directory_path
                            .read_dir()
                            .map_err(|e| Error::DirectoryRead(directory_path.clone(), e))?
                        {
                            let entry_name = entry
                                .map_err(|e| Error::DirectoryRead(directory_path.clone(), e))?
                                .file_name();

                            directory_contents.insert(entry_name);
                        }

                        for (action_name, products) in &action_products {
                            if products
                                .iter()
                                .all(|p| directory_contents.contains(OsStr::new(&p)))
                            {
                                sender.send((current_directory.clone(), action_name.clone()))?;
                            }
                        }

                        progress.inc(1);
                        directory_path.pop();
                        directory_contents.clear();
                    }
                });

        threads.push(handle.expect("Should be able to spawn threads."));
    }

    CompletedDirectories {
        threads,
        receiver,
        progress: progress.clone(),
    }
}

impl CompletedDirectories {
    /// Get the directories that have been completed for each action.
    ///
    /// # Errors
    /// Returns `Err<row::Error>` when the workspace directories cannot be accessed.
    ///
    /// # Panics
    /// This method should not panic.
    ///
    pub fn get(self) -> Result<HashMap<String, HashSet<PathBuf>>, Error> {
        let mut result = HashMap::new();
        for (directory, action) in &self.receiver {
            result
                .entry(action)
                .or_insert(HashSet::new())
                .insert(directory);
        }

        for handle in self.threads {
            handle.join().expect("The thread should not panic")?;
        }

        self.progress.finish();

        Ok(result)
    }
}

/// JSON values of directories.
///
/// Call `get()` to wait for all pending threads to complete and return the result.
///
pub(crate) struct DirectoryValues {
    /// Threads reading the values.
    threads: Vec<JoinHandle<Result<(), Error>>>,

    /// Channel to receive results from worker threads.
    receiver: Receiver<(PathBuf, Value)>,

    /// Progress bar.
    progress: ProgressBar,
}

/// Read value files from directories.
///
/// `read_values` spawns threads that read the JSON value files and
/// returns immediately. Calling `get` on the result will wait for the threads
/// to complete and then provides the map of directory names to values.
///
/// # Arguments
/// * `workflow` - The `Workflow` to read from.
/// * `directories` - The directories to read. Must be present in the workspace.
/// * `io_threads` - Number of threads to use while scanning directories.
///
pub(crate) fn read_values(
    workflow: &Workflow,
    directories: Vec<PathBuf>,
    io_threads: u16,
    multi_progress: &mut MultiProgressContainer,
) -> DirectoryValues {
    let (sender, receiver) = mpsc::channel();

    let mut progress = ProgressBar::new(directories.len() as u64).with_message("Reading values");
    progress = multi_progress.add_or_hide(progress, directories.len() < MIN_PROGRESS_BAR_SIZE);
    progress.set_style(progress_styles::counted_bar());
    progress.tick();

    if !directories.is_empty() {
        debug!("Reading directory values.");
    }

    let workspace_path = workflow.root.join(&workflow.workspace.path);
    let directories_mutex = Arc::new(Mutex::new(directories));

    let mut threads = Vec::with_capacity(io_threads as usize);

    for i in 0..io_threads {
        let workspace_path = workspace_path.clone();
        let directories_mutex = directories_mutex.clone();
        let sender = sender.clone();
        let progress = progress.clone();
        let value_file = workflow.workspace.value_file.clone();

        let thread_name = format!("read-values-{i}");
        let handle =
            thread::Builder::new()
                .name(thread_name)
                .spawn(move || -> Result<(), Error> {
                    let mut value_path = workspace_path;

                    loop {
                        let current_directory;

                        // Pull the next directory to process off the shared stack.
                        {
                            let mut directories = directories_mutex.lock().unwrap();
                            if let Some(d) = directories.pop() {
                                current_directory = d;
                            } else {
                                break Ok(());
                            }
                        }

                        // List all files in the current directory.
                        value_path.push(&current_directory);

                        // Parse the value JSON file (if given).
                        if let Some(ref value_file) = value_file {
                            value_path.push(value_file);

                            let value_str = fs::read_to_string(&value_path)
                                .map_err(|e| Error::FileRead(value_path.clone(), e))?;
                            let value: Value = serde_json::from_str(&value_str)
                                .map_err(|e| Error::JSONParse(value_path.clone(), e))?;

                            sender.send((current_directory.clone(), value))?;

                            value_path.pop();
                        } else {
                            sender.send((current_directory.clone(), Value::Null))?;
                        }

                        progress.inc(1);
                        value_path.pop();
                    }
                });

        threads.push(handle.expect("Should be able to spawn threads."));
    }

    DirectoryValues {
        threads,
        receiver,
        progress: progress.clone(),
    }
}

impl DirectoryValues {
    /// Get the JSON value of each directory.
    pub(crate) fn get(self) -> Result<HashMap<PathBuf, Value>, Error> {
        let mut result: HashMap<PathBuf, Value> = HashMap::new();
        for (directory, value) in &self.receiver {
            result.entry(directory).or_insert(value);
        }

        for handle in self.threads {
            handle.join().expect("The thread should not panic")?;
        }

        self.progress.finish();

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use assert_fs::prelude::*;
    use assert_fs::TempDir;
    use indicatif::{MultiProgress, ProgressDrawTarget};
    use serial_test::parallel;
    use std::path::PathBuf;

    use super::*;
    use crate::workflow::Workflow;

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
    fn list() {
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

        let result = list_directories(&workflow, &mut multi_progress).unwrap();
        assert!(result.contains(&PathBuf::from("dir1")));
        assert!(result.contains(&PathBuf::from("dir2")));
        assert!(result.contains(&PathBuf::from("dir3")));
    }

    #[test]
    #[parallel]
    fn find_completed() {
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

        let workflow = r#"
[[action]]
name = "one"
command = "c"
products = ["1"]

[[action]]
name = "two"
command = "c"
products = ["2"]

[[action]]
name = "three"
command = "c"
products = ["3", "4"]
"#;

        temp.child("workspace")
            .child("dir1")
            .child("1")
            .touch()
            .unwrap();
        temp.child("workspace")
            .child("dir2")
            .child("2")
            .touch()
            .unwrap();
        temp.child("workspace")
            .child("dir3")
            .child("1")
            .touch()
            .unwrap();
        temp.child("workspace")
            .child("dir3")
            .child("2")
            .touch()
            .unwrap();
        temp.child("workspace")
            .child("dir4")
            .child("3")
            .touch()
            .unwrap();
        temp.child("workspace")
            .child("dir4")
            .child("4")
            .touch()
            .unwrap();
        temp.child("workspace")
            .child("dir5")
            .child("3")
            .touch()
            .unwrap();

        let workflow = Workflow::open_str(temp.path(), workflow).unwrap();

        let result = find_completed_directories(
            &workflow,
            vec![
                PathBuf::from("dir1"),
                PathBuf::from("dir2"),
                PathBuf::from("dir3"),
                PathBuf::from("dir4"),
                PathBuf::from("dir5"),
            ],
            2,
            &mut multi_progress,
        )
        .get()
        .unwrap();

        assert!(result.contains_key("one"));
        assert_eq!(result["one"].len(), 2);
        assert!(result["one"].contains(&PathBuf::from("dir1")));
        assert!(result["one"].contains(&PathBuf::from("dir3")));
        assert!(result.contains_key("two"));
        assert_eq!(result["two"].len(), 2);
        assert!(result["two"].contains(&PathBuf::from("dir2")));
        assert!(result["two"].contains(&PathBuf::from("dir3")));
        assert!(result["three"].contains(&PathBuf::from("dir4")));

        assert!(!result.contains_key("four"));
    }

    #[test]
    #[parallel]
    fn read() {
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

        let workflow = r#"
[workspace]
value_file = "v"
"#;

        temp.child("workspace")
            .child("dir1")
            .child("v")
            .write_str("1")
            .unwrap();
        temp.child("workspace")
            .child("dir2")
            .child("v")
            .write_str("2")
            .unwrap();
        temp.child("workspace")
            .child("dir3")
            .child("v")
            .write_str("3")
            .unwrap();

        let workflow = Workflow::open_str(temp.path(), workflow).unwrap();

        let result = read_values(
            &workflow,
            vec![
                PathBuf::from("dir1"),
                PathBuf::from("dir2"),
                PathBuf::from("dir3"),
            ],
            2,
            &mut multi_progress,
        )
        .get()
        .unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[&PathBuf::from("dir1")].as_i64(), Some(1));
        assert_eq!(result[&PathBuf::from("dir2")].as_i64(), Some(2));
        assert_eq!(result[&PathBuf::from("dir3")].as_i64(), Some(3));
    }
}
