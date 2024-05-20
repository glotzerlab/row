// Copyright (c) 2024 The Regents of the University of Michigan.
// Part of row, released under the BSD 3-Clause License.

use indicatif::ProgressBar;
use log::{debug, trace, warn};
use serde_json::Value;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use crate::cluster::{self, SchedulerType};
use crate::expr;
use crate::launcher;
use crate::progress_styles;
use crate::scheduler::bash::Bash;
use crate::scheduler::slurm::Slurm;
use crate::scheduler::Scheduler;
use crate::state::State;
use crate::workflow::{Action, Selector, Workflow};
use crate::{Error, MultiProgressContainer};

/// Encapsulate the workflow, state, and scheduler into a project.
///
/// When opened, `Project`:
///
/// * Reads caches from disk.
/// * Gets the status of submitted jobs from the scheduler.
/// * Collects the staged completions.
/// * Reads the workflow file
/// * Synchronizes the system state with the workspace on disk.
/// * And removes any completed jobs from the submitted cache.
///
/// These are common operations used by many CLI commands. A command that needs
/// only a subset of these should use the individual classes directly.
///
pub struct Project {
    /// The project's workflow definition.
    workflow: Workflow,

    /// The state associate with the directories in the project.
    state: State,

    /// The scheduler.
    scheduler: Box<dyn Scheduler>,

    /// The cluster's name.
    cluster_name: String,
}

/// Store individual sets of jobs, separated by status for a given action.
///
/// Call `Project::separate_by_status` to produce a `Status`.
///
#[derive(Debug)]
pub struct Status {
    /// Directories that have completed.
    pub completed: Vec<PathBuf>,

    /// Directories that have been submitted to the scheduler.
    pub submitted: Vec<PathBuf>,

    /// Directories that are eligible to execute.
    pub eligible: Vec<PathBuf>,

    /// Directories that are waiting on previous actions to complete.
    pub waiting: Vec<PathBuf>,
}

impl Project {
    /// Open a project from the current working directory or any parents.
    ///
    /// # Errors
    /// Returns `Err<row::Error>` when the project cannot be opened.
    ///
    pub fn open(
        io_threads: u16,
        cluster_name: &Option<String>,
        multi_progress: &mut MultiProgressContainer,
    ) -> Result<Project, Error> {
        trace!("Opening project.");
        let workflow = Workflow::open()?;
        let clusters = cluster::Configuration::open()?;
        let cluster = clusters.identify(cluster_name.as_deref())?;
        let launchers = launcher::Configuration::open()?.by_cluster(&cluster.name);
        let cluster_name = cluster.name.clone();

        let scheduler: Box<dyn Scheduler> = match cluster.scheduler {
            SchedulerType::Bash => Box::new(Bash::new(cluster, launchers)),
            SchedulerType::Slurm => Box::new(Slurm::new(cluster, launchers)),
        };

        let mut state = State::from_cache(&workflow)?;

        // squeue will likely take the longest to finish, start it first.
        let jobs = state.jobs_submitted_on(&cluster_name);
        let mut progress =
            ProgressBar::new_spinner().with_message("Checking submitted job statuses");
        progress = multi_progress.add_or_hide(progress, jobs.is_empty());

        progress.enable_steady_tick(Duration::from_millis(progress_styles::STEADY_TICK));
        progress.set_style(progress_styles::uncounted_spinner());
        progress.tick();

        let active_jobs = scheduler.active_jobs(&jobs)?;

        // Then synchronize with the workspace while squeue is running.
        state.synchronize_workspace(&workflow, io_threads, multi_progress)?;

        // Now, wait for squeue to finish and remove any inactive jobs.
        let active_jobs = active_jobs.get()?;
        progress.finish();

        if active_jobs.len() != jobs.len() {
            state.remove_inactive_submitted(&cluster_name, &active_jobs);
        } else if !jobs.is_empty() {
            trace!("All submitted jobs remain active on {cluster_name}.");
        }

        Ok(Self {
            workflow,
            state,
            scheduler,
            cluster_name,
        })
    }

    /// Close the project.
    ///
    /// Closing saves the updated cache to disk and removes any temporary
    /// completion pack files. `Project` does not automatically close when
    /// dropped as the caller may not wish to save the cache when there is
    /// an error after opening the project.
    ///
    /// # Errors
    /// Returns `Err<row::Error>` when there is an error taking these steps.
    ///
    pub fn close(&mut self, multi_progress: &mut MultiProgressContainer) -> Result<(), Error> {
        debug!("Closing project.");

        self.state.save_cache(&self.workflow, multi_progress)?;

        Ok(())
    }

    /// Get the project's workflow definition.
    pub fn workflow(&self) -> &Workflow {
        &self.workflow
    }

    /// Get the state of the project's workspace.
    pub fn state(&self) -> &State {
        &self.state
    }

    /// Find the directories that are included by the action.
    ///
    /// # Parameters:
    /// - `action`: The action to match.
    /// - `directories`: Directories to match.
    ///
    /// # Returns
    /// `Ok(Vec<PathBuf>)` listing directories from `directories` that match
    /// the action's **include** directive.
    ///
    /// # Errors
    /// `Err(row::Error)` when any action's include pointer cannot be resolved.
    ///
    /// # Warnings
    /// Logs with `warn!` when `subset` contains directories that are not
    /// present in the workspace.
    ///
    pub fn find_matching_directories(
        &self,
        action: &Action,
        directories: Vec<PathBuf>,
    ) -> Result<Vec<PathBuf>, Error> {
        trace!(
            "Finding directories that action '{}' includes.",
            action.name()
        );

        let mut matching_directories = Vec::with_capacity(directories.len());

        'outer: for name in directories {
            if let Some(value) = self.state.values().get(&name) {
                if action.group.include().is_empty() {
                    matching_directories.push(name);
                } else {
                    for selector in action.group.include() {
                        let result = match selector {
                            Selector::Condition((include, comparison, expected)) => {
                                let actual = value.pointer(include).ok_or_else(|| {
                                    Error::JSONPointerNotFound(name.clone(), include.clone())
                                })?;

                                expr::evaluate_json_comparison(comparison, actual, expected)
                                    .ok_or_else(|| {
                                        Error::CannotCompareInclude(
                                            actual.clone(),
                                            expected.clone(),
                                            name.clone(),
                                        )
                                    })
                            }

                            Selector::All(conditions) => {
                                let mut matches = 0;
                                for (include, comparison, expected) in conditions {
                                    let actual = value.pointer(include).ok_or_else(|| {
                                        Error::JSONPointerNotFound(name.clone(), include.clone())
                                    })?;

                                    if expr::evaluate_json_comparison(comparison, actual, expected)
                                        .ok_or_else(|| {
                                            Error::CannotCompareInclude(
                                                actual.clone(),
                                                expected.clone(),
                                                name.clone(),
                                            )
                                        })?
                                    {
                                        matches += 1;
                                    }
                                }
                                Ok(matches == conditions.len())
                            }
                        };

                        if result? {
                            matching_directories.push(name);
                            continue 'outer;
                        }
                    }
                }
            } else {
                warn!("Directory '{}' not found in workspace.", name.display());
            }
        }

        trace!("Found {} match(es).", matching_directories.len());
        Ok(matching_directories)
    }

    /// Separate a set of directories by their status.
    ///
    /// # Parameters:
    /// - `action`: Report the status for this action.
    /// - `directories`: Directories to separate.
    ///
    /// # Returns
    /// `Ok(Status)` listing all input `directories` in categories.
    ///
    /// # Errors
    /// `Err(row::Error)` when a given directory is not present.
    ///
    pub fn separate_by_status(
        &self,
        action: &Action,
        directories: Vec<PathBuf>,
    ) -> Result<Status, Error> {
        trace!(
            "Separating {} directories by status for '{}'.",
            directories.len(),
            action.name()
        );
        let capacity = directories.capacity();
        let mut status = Status {
            completed: Vec::with_capacity(capacity),
            submitted: Vec::with_capacity(capacity),
            eligible: Vec::with_capacity(capacity),
            waiting: Vec::with_capacity(capacity),
        };

        for directory_name in directories {
            if !self.state.values().contains_key(&directory_name) {
                return Err(Error::DirectoryNotFound(directory_name));
            }

            let completed = self.state.completed();

            if completed[action.name()].contains(&directory_name) {
                status.completed.push(directory_name);
            } else if self.state.is_submitted(action.name(), &directory_name) {
                status.submitted.push(directory_name);
            } else if action
                .previous_actions()
                .iter()
                .all(|a| completed[a].contains(&directory_name))
            {
                status.eligible.push(directory_name);
            } else {
                status.waiting.push(directory_name);
            }
        }

        Ok(status)
    }

    /// Separate directories into groups based on the given parameters
    ///
    /// # Errors
    /// `Err(row::Error)` when a given directory is not present or a JSON
    /// pointer used for sorting is not present.
    ///
    /// # Panics
    /// When two JSON pointers are not valid for comparison.
    ///
    pub fn separate_into_groups(
        &self,
        action: &Action,
        mut directories: Vec<PathBuf>,
    ) -> Result<Vec<Vec<PathBuf>>, Error> {
        trace!(
            "Separating {} directories into groups for '{}'.",
            directories.len(),
            action.name()
        );

        if directories.is_empty() {
            return Ok(Vec::new());
        }

        // First, sort the directories by name.
        directories.sort_unstable();

        // Determine the user-provided sort keys.
        let mut sort_keys = HashMap::new();
        for directory_name in &directories {
            let value = self
                .state
                .values()
                .get(directory_name)
                .ok_or_else(|| Error::DirectoryNotFound(directory_name.clone()))?;

            let mut sort_key = Vec::new();
            for pointer in action.group.sort_by() {
                let element = value.pointer(pointer).ok_or_else(|| {
                    Error::JSONPointerNotFound(directory_name.clone(), pointer.clone())
                })?;
                sort_key.push(element.clone());
            }
            sort_keys.insert(directory_name.clone(), Value::Array(sort_key));
        }

        // Sort by key when there are keys to sort by.
        let mut result = Vec::new();
        if action.group.sort_by().is_empty() {
            if action.group.reverse_sort() {
                directories.reverse();
            }
            result.push(directories);
        } else {
            directories.sort_by(|a, b| {
                expr::partial_cmp_json_values(&sort_keys[a], &sort_keys[b])
                    .expect("Valid JSON comparison")
            });

            if action.group.reverse_sort() {
                directories.reverse();
            }

            // Split by the sort key when requested.
            #[allow(clippy::redundant_closure_for_method_calls)]
            if action.group.split_by_sort_key() {
                result.extend(
                    directories
                        .chunk_by(|a, b| {
                            expr::partial_cmp_json_values(&sort_keys[a], &sort_keys[b])
                                .expect("Valid JSON comparison")
                                == Ordering::Equal
                        })
                        .map(|v| v.to_vec()),
                );
            } else {
                result.push(directories);
            }
        }

        if let Some(maximum_size) = action.group.maximum_size {
            let mut new_result = Vec::new();
            for array in result {
                #[allow(clippy::redundant_closure_for_method_calls)]
                new_result.extend(array.chunks(maximum_size).map(|v| v.to_vec()));
            }

            result = new_result;
        }

        Ok(result)
    }

    /// Get the scheduler.
    pub fn scheduler(&self) -> &dyn Scheduler {
        self.scheduler.as_ref()
    }

    /// Add a new submitted job.
    pub fn add_submitted(&mut self, action_name: &str, directories: &[PathBuf], job_id: u32) {
        self.state
            .add_submitted(action_name, directories, &self.cluster_name, job_id);
    }
}

#[cfg(test)]
mod tests {
    use assert_fs::prelude::*;
    use assert_fs::TempDir;
    use indicatif::{MultiProgress, ProgressDrawTarget};
    use serde_json::Value;
    use serial_test::serial;
    use std::env;

    use super::*;
    use crate::workflow::Comparison;

    fn setup(n: usize) -> Project {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::max())
            .is_test(true)
            .try_init();

        let multi_progress = MultiProgress::with_draw_target(ProgressDrawTarget::hidden());
        let mut multi_progress = MultiProgressContainer {
            progress_bars: Vec::new(),
            multi_progress,
        };

        let temp = TempDir::new().unwrap();
        env::set_current_dir(temp.path()).unwrap();
        for i in 0..n {
            let directory = temp.child("workspace").child(format!("dir{i}"));
            directory.create_dir_all().unwrap();
            directory
                .child("v")
                .write_str(&format!(r#"{{"i": {}, "j": {}}}"#, i, (n - 1 - i) / 2))
                .unwrap();

            if i < n / 2 {
                directory.child("two").touch().unwrap();
            }
            directory.child("one").touch().unwrap();
        }

        let workflow = format!(
            r#"
[workspace]
value_file = "v"

[[action]]
name = "one"
command = "c"
products = ["one"]

[[action]]
name = "two"
command = "c"
products = ["two"]
[[action.group.include]]
condition = ["/i", "<", {}]

[[action]]
name = "three"
command = "c"
products = ["three"]
previous_actions = ["two"]
"#,
            n - 2
        );

        temp.child("workflow.toml").write_str(&workflow).unwrap();

        Project::open(2, &None, &mut multi_progress).unwrap()
    }

    #[test]
    #[serial]
    fn matching() {
        let project = setup(8);

        let mut all_directories = project.state().list_directories();
        all_directories.sort_unstable();

        let action = &project.workflow.action[0];
        assert_eq!(
            project
                .find_matching_directories(action, all_directories.clone())
                .unwrap(),
            all_directories[0..8]
        );

        let action = &project.workflow.action[1];
        assert_eq!(
            project
                .find_matching_directories(action, all_directories.clone())
                .unwrap(),
            all_directories[0..6]
        );

        // Check all conditions.
        let mut action = project.workflow.action[1].clone();
        let include = action.group.include.as_mut().unwrap();
        include.clear();
        include.push(Selector::All(vec![
            ("/i".into(), Comparison::GreaterThan, Value::from(4)),
            ("/i".into(), Comparison::LessThan, Value::from(6)),
        ]));
        assert_eq!(
            project
                .find_matching_directories(&action, all_directories.clone())
                .unwrap(),
            vec![PathBuf::from("dir5")]
        );

        // TODO, test any
    }

    #[test]
    #[serial]
    fn status() {
        let project = setup(8);

        let mut all_directories = project.state().list_directories();
        all_directories.sort_unstable();

        let action = &project.workflow.action[0];
        let status = project
            .separate_by_status(action, all_directories.clone())
            .unwrap();
        assert_eq!(status.completed, all_directories);
        assert!(status.submitted.is_empty());
        assert!(status.eligible.is_empty());
        assert!(status.waiting.is_empty());

        let action = &project.workflow.action[1];
        let status = project
            .separate_by_status(action, all_directories.clone())
            .unwrap();
        assert_eq!(status.completed, all_directories[0..4]);
        assert!(status.submitted.is_empty());
        assert_eq!(status.eligible, all_directories[4..8]);
        assert!(status.waiting.is_empty());

        let action = &project.workflow.action[2];
        let status = project
            .separate_by_status(action, all_directories.clone())
            .unwrap();
        assert!(status.completed.is_empty());
        assert!(status.submitted.is_empty());
        assert_eq!(status.eligible, all_directories[0..4]);
        assert_eq!(status.waiting, all_directories[4..8]);
    }

    #[test]
    #[serial]
    fn group() {
        let project = setup(8);

        let mut all_directories = project.state().list_directories();
        all_directories.sort_unstable();

        let action = &project.workflow.action[0];
        let groups = project
            .separate_into_groups(action, all_directories.clone())
            .unwrap();
        assert_eq!(groups, vec![all_directories]);
    }

    #[test]
    #[serial]
    fn group_reverse() {
        let project = setup(8);

        let mut all_directories = project.state().list_directories();
        all_directories.sort_unstable();
        let mut reversed = all_directories.clone();
        reversed.reverse();

        let mut action = project.workflow.action[0].clone();
        action.group.reverse_sort = Some(true);
        let groups = project
            .separate_into_groups(&action, all_directories.clone())
            .unwrap();
        assert_eq!(groups, vec![reversed]);
    }

    #[test]
    #[serial]
    fn group_max_size() {
        let project = setup(8);

        let mut all_directories = project.state().list_directories();
        all_directories.sort_unstable();

        let mut action = project.workflow.action[0].clone();
        action.group.maximum_size = Some(3);
        let groups = project
            .separate_into_groups(&action, all_directories.clone())
            .unwrap();
        assert_eq!(
            groups,
            vec![
                all_directories[0..3].to_vec(),
                all_directories[3..6].to_vec(),
                all_directories[6..8].to_vec()
            ]
        );
    }

    #[test]
    #[serial]
    fn group_sort() {
        let project = setup(8);

        let mut all_directories = project.state().list_directories();
        all_directories.sort_unstable();

        let mut action = project.workflow.action[0].clone();
        action.group.sort_by = Some(vec!["/j".to_string()]);
        let groups = project
            .separate_into_groups(&action, all_directories.clone())
            .unwrap();
        assert_eq!(
            groups,
            vec![vec![
                PathBuf::from("dir6"),
                PathBuf::from("dir7"),
                PathBuf::from("dir4"),
                PathBuf::from("dir5"),
                PathBuf::from("dir2"),
                PathBuf::from("dir3"),
                PathBuf::from("dir0"),
                PathBuf::from("dir1")
            ]]
        );
    }

    #[test]
    #[serial]
    fn group_sort_and_split() {
        let project = setup(8);

        let mut all_directories = project.state().list_directories();
        all_directories.sort_unstable();

        let mut action = project.workflow.action[0].clone();
        action.group.sort_by = Some(vec!["/j".to_string()]);
        action.group.split_by_sort_key = Some(true);
        let groups = project
            .separate_into_groups(&action, all_directories.clone())
            .unwrap();
        assert_eq!(
            groups,
            vec![
                vec![PathBuf::from("dir6"), PathBuf::from("dir7")],
                vec![PathBuf::from("dir4"), PathBuf::from("dir5")],
                vec![PathBuf::from("dir2"), PathBuf::from("dir3")],
                vec![PathBuf::from("dir0"), PathBuf::from("dir1")]
            ]
        );
    }
}
