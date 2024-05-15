// Copyright (c) 2024 The Regents of the University of Michigan.
// Part of row, released under the BSD 3-Clause License.

use human_format::Formatter;
use log::{debug, trace, warn};
use serde::{Deserialize, Deserializer};
use serde_json;
use speedate::Duration;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader};
use std::ops::Add;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::Error;

/// The workflow definition.
///
/// `Workflow` is the in-memory realization of the user provided `workflow.toml`.
///
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Workflow {
    /// The root directory of the row project (absolute).
    #[serde(skip)]
    pub root: PathBuf,

    /// The workspace parameters.
    #[serde(default)]
    pub workspace: Workspace,

    /// Default tables
    #[serde(default)]
    pub default: DefaultTables,

    /// The actions.
    #[serde(default)]
    pub action: Vec<Action>,
}

/// The workspace definition.
///
/// `Workspace` stores the user-provided options defining the workspace.
///
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Workspace {
    /// The workspace directory
    #[serde(default = "default_workspace_path")]
    pub path: PathBuf,

    /// Names of the static value file.
    pub value_file: Option<PathBuf>,
}

/// The submission options
///
/// `SubmitOPtions` stores the user-provided cluster specific submission options for a workflow or
/// action.
///
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SubmitOptions {
    /// The account.
    pub account: Option<String>,

    /// Setup commands.
    pub setup: Option<String>,

    /// Custom options.
    #[serde(default)]
    pub custom: Vec<String>,

    /// The partition.
    pub partition: Option<String>,
}

/// The action definition.
///
/// `Action` stores the user-provided options for a given action.
///
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Action {
    /// Unique name defining the action.
    pub name: Option<String>,

    /// The command to execute for this action.
    pub command: Option<String>,

    /// Names of the launchers to use when executing the action.
    #[serde(default)]
    pub launchers: Option<Vec<String>>,

    /// The names of the previous actions that must be completed before this action.
    #[serde(default)]
    pub previous_actions: Option<Vec<String>>,

    /// The product files this action creates.
    #[serde(default)]
    pub products: Option<Vec<String>>,

    /// Resources used by this action.
    #[serde(default)]
    pub resources: Resources,

    /// The cluster specific submission options.
    #[serde(default)]
    pub submit_options: HashMap<String, SubmitOptions>,

    /// The group of jobs to submit.
    #[serde(default)]
    pub group: Group,

    // Name of the group to copy defaults from.
    pub from: Option<String>,
}

/// Default tables
///
/// Store default options for other tables in the file.
///
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DefaultTables {
    #[serde(default)]
    pub action: Action,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Walltime {
    #[serde(deserialize_with = "deserialize_duration_from_str")]
    PerSubmission(Duration),
    #[serde(deserialize_with = "deserialize_duration_from_str")]
    PerDirectory(Duration),
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Processes {
    PerSubmission(usize),
    PerDirectory(usize),
}

/// Resources used by an action.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Resources {
    /// Number of processes.
    pub processes: Option<Processes>,

    /// Threads per process.
    pub threads_per_process: Option<usize>,

    /// GPUs per process.
    pub gpus_per_process: Option<usize>,

    // Walltime.
    pub walltime: Option<Walltime>,
}

/// Comparison operations
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Comparison {
    #[serde(rename(deserialize = "<"))]
    LessThan,
    #[serde(rename(deserialize = "<="))]
    LessThanOrEqualTo,
    #[serde(rename(deserialize = "=="))]
    EqualTo,
    #[serde(rename(deserialize = ">="))]
    GreaterThanOrEqualTo,
    #[serde(rename(deserialize = ">"))]
    GreaterThan,
}

/// Group definition.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Group {
    /// Include members of the group where all JSON elements match the given values.
    #[serde(default)]
    pub include: Option<Vec<(String, Comparison, serde_json::Value)>>,

    /// Sort by the given set of JSON elements.
    #[serde(default)]
    pub sort_by: Option<Vec<String>>,

    /// Split into groups by the sort keys.
    #[serde(default)]
    pub split_by_sort_key: Option<bool>,

    /// Reverse the sort.
    #[serde(default)]
    pub reverse_sort: Option<bool>,

    /// Maximum size of the submitted group.
    pub maximum_size: Option<usize>,

    /// Submit only whole groups when true.
    #[serde(default)]
    pub submit_whole: Option<bool>,
}

/// Resource cost to execute an action.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ResourceCost {
    /// Number of CPU hours.
    pub cpu_hours: f64,
    /// Number of GPU hours.
    pub gpu_hours: f64,
}

impl Default for Walltime {
    fn default() -> Self {
        Self::PerDirectory(
            Duration::new(true, 0, 3600, 0).expect("3600 seconds is a valid duration"),
        )
    }
}

impl Default for Processes {
    fn default() -> Self {
        Self::PerSubmission(1)
    }
}

impl ResourceCost {
    /// Create a zero-valued `ResourceCost`
    pub fn new() -> Self {
        Self {
            cpu_hours: 0.0,
            gpu_hours: 0.0,
        }
    }

    /// Create a new `ResourceCost`.
    #[allow(clippy::similar_names)]
    pub fn with_values(cpu_hours: f64, gpu_hours: f64) -> Self {
        Self {
            cpu_hours,
            gpu_hours,
        }
    }

    /// Check if the cost is exactly 0
    pub fn is_zero(&self) -> bool {
        self.cpu_hours == 0.0 && self.gpu_hours == 0.0
    }
}

impl fmt::Display for ResourceCost {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut formatter = Formatter::new();
        formatter.with_decimals(0);
        formatter.with_separator("");

        if self.gpu_hours != 0.0 && self.cpu_hours != 0.0 {
            write!(
                f,
                "{} CPU-hours and {} GPU-hours",
                formatter.format(self.cpu_hours),
                formatter.format(self.gpu_hours)
            )
        } else if self.gpu_hours != 0.0 && self.cpu_hours == 0.0 {
            write!(f, "{} GPU-hours", formatter.format(self.gpu_hours))
        } else {
            write!(f, "{} CPU-hours", formatter.format(self.cpu_hours))
        }
    }
}

impl Add for ResourceCost {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            cpu_hours: self.cpu_hours + other.cpu_hours,
            gpu_hours: self.gpu_hours + other.gpu_hours,
        }
    }
}

impl Resources {
    /// Determine the total number of processes this action will use.
    ///
    /// # Arguments
    /// `n_directories`: Number of directories in the submission.
    ///
    pub fn total_processes(&self, n_directories: usize) -> usize {
        match self.processes() {
            Processes::PerDirectory(p) => p * n_directories,
            Processes::PerSubmission(p) => p,
        }
    }

    /// Determine the total number of CPUs this action will use.
    ///
    /// # Arguments
    /// `n_directories`: Number of directories in the submission.
    ///
    pub fn total_cpus(&self, n_directories: usize) -> usize {
        self.total_processes(n_directories) * self.threads_per_process.unwrap_or(1)
    }

    /// Determine the total number of GPUs this action will use.
    ///
    /// # Arguments
    /// `n_directories`: Number of directories in the submission.
    ///
    pub fn total_gpus(&self, n_directories: usize) -> usize {
        self.total_processes(n_directories) * self.gpus_per_process.unwrap_or(0)
    }

    /// Determine the total walltime this action will use.
    ///
    /// # Arguments
    /// `n_directories`: Number of directories in the submission.
    ///
    /// # Panics
    /// When the resulting walltime cannot be represented.
    ///
    pub fn total_walltime(&self, n_directories: usize) -> Duration {
        match self.walltime() {
            Walltime::PerDirectory(ref w) => Duration::new(
                true,
                0,
                (w.signed_total_seconds() * (n_directories as i64)) as u32,
                0,
            )
            .expect("Valid duration."),
            Walltime::PerSubmission(ref w) => w.clone(),
        }
    }

    /// Compute the total resource usage of an action execution.
    ///
    /// The cost is computed assuming that every job is executed to the full
    /// requested walltime.
    ///
    pub fn cost(&self, n_directories: usize) -> ResourceCost {
        let process_hours = ((self.total_processes(n_directories) as i64)
            * self.total_walltime(n_directories).signed_total_seconds())
            as f64
            / 3600.0;

        if let Some(gpus_per_process) = self.gpus_per_process {
            return ResourceCost {
                gpu_hours: process_hours * gpus_per_process as f64,
                cpu_hours: 0.0,
            };
        }

        if let Some(threads_per_process) = self.threads_per_process {
            return ResourceCost {
                cpu_hours: process_hours * threads_per_process as f64,
                gpu_hours: 0.0,
            };
        }

        ResourceCost {
            cpu_hours: process_hours,
            gpu_hours: 0.0,
        }
    }

    /// Resolve omitted keys from the given template.
    fn resolve(&mut self, template: &Resources) {
        if self.processes.is_none() {
            self.processes.clone_from(&template.processes);
        }
        if self.threads_per_process.is_none() {
            self.threads_per_process = template.threads_per_process;
        }
        if self.gpus_per_process.is_none() {
            self.gpus_per_process = template.gpus_per_process;
        }
        if self.walltime.is_none() {
            self.walltime.clone_from(&template.walltime);
        }
    }

    pub fn processes(&self) -> Processes {
        if let Some(processes) = self.processes.as_ref() {
            processes.clone()
        } else {
            Processes::default()
        }
    }
    pub fn walltime(&self) -> Walltime {
        if let Some(walltime) = self.walltime.as_ref() {
            walltime.clone()
        } else {
            Walltime::default()
        }
    }
}

impl Action {
    /// Get the action's `name`.
    pub fn name(&self) -> &str {
        if let Some(name) = self.name.as_ref() {
            name
        } else {
            ""
        }
    }

    /// Get the action's `command`.
    pub fn command(&self) -> &str {
        if let Some(command) = self.command.as_ref() {
            command
        } else {
            ""
        }
    }

    /// Get the action's `launchers`.
    pub fn launchers(&self) -> &[String] {
        if let Some(launchers) = self.launchers.as_ref() {
            launchers
        } else {
            &[]
        }
    }

    /// Get the action's `previous_actions`.
    pub fn previous_actions(&self) -> &[String] {
        if let Some(previous_actions) = self.previous_actions.as_ref() {
            previous_actions
        } else {
            &[]
        }
    }

    /// Get the action's products
    pub fn products(&self) -> &[String] {
        if let Some(products) = self.products.as_ref() {
            products
        } else {
            &[]
        }
    }

    /// Resolve the action's omitted keys with defaults
    fn resolve(&mut self, template: &Action) {
        if self.name.is_none() {
            self.name.clone_from(&template.name);
        }
        if self.command.is_none() {
            self.command.clone_from(&template.command);
        }
        if self.launchers.is_none() {
            self.launchers.clone_from(&template.launchers);
        }
        if self.previous_actions.is_none() {
            self.previous_actions.clone_from(&template.previous_actions);
        }
        if self.products.is_none() {
            self.products.clone_from(&template.products);
        }

        self.resources.resolve(&template.resources);
        self.group.resolve(&template.group);

        // Populate each action's submit_options with the global ones.
        for (name, template_options) in &template.submit_options {
            if self.submit_options.contains_key(name) {
                let action_options = self
                    .submit_options
                    .get_mut(name)
                    .expect("Key should be present");
                if action_options.account.is_none() {
                    action_options.account.clone_from(&template_options.account);
                }
                if action_options.setup.is_none() {
                    action_options.setup.clone_from(&template_options.setup);
                }
                if action_options.partition.is_none() {
                    action_options
                        .partition
                        .clone_from(&template_options.partition);
                }
                if action_options.custom.is_empty() {
                    action_options.custom.clone_from(&template_options.custom);
                }
            } else {
                self.submit_options
                    .insert(name.clone(), template_options.clone());
            }
        }
    }
}

impl Group {
    /// Get the group's `include`.
    pub fn include(&self) -> &[(String, Comparison, serde_json::Value)] {
        if let Some(include) = self.include.as_ref() {
            include
        } else {
            &[]
        }
    }

    /// Get the group's `sort_by`.
    pub fn sort_by(&self) -> &[String] {
        if let Some(sort_by) = self.sort_by.as_ref() {
            sort_by
        } else {
            &[]
        }
    }

    /// Get the group's `split_by_sort_key`.
    pub fn split_by_sort_key(&self) -> bool {
        if let Some(split_by_sort_key) = self.split_by_sort_key {
            split_by_sort_key
        } else {
            false
        }
    }

    /// Get the group's `reverse_sort`.
    pub fn reverse_sort(&self) -> bool {
        if let Some(reverse_sort) = self.reverse_sort {
            reverse_sort
        } else {
            false
        }
    }

    /// Get the group's `submit_whole`.
    pub fn submit_whole(&self) -> bool {
        if let Some(submit_whole) = self.submit_whole {
            submit_whole
        } else {
            false
        }
    }

    /// Resolve omitted keys from the given template.
    fn resolve(&mut self, template: &Group) {
        if self.include.is_none() {
            self.include.clone_from(&template.include);
        }
        if self.sort_by.is_none() {
            self.sort_by.clone_from(&template.sort_by);
        }
        if self.split_by_sort_key.is_none() {
            self.split_by_sort_key = template.split_by_sort_key;
        }
        if self.reverse_sort.is_none() {
            self.reverse_sort = template.reverse_sort;
        }
        if self.maximum_size.is_none() {
            self.maximum_size = template.maximum_size;
        }
        if self.submit_whole.is_none() {
            self.submit_whole = template.submit_whole;
        }
    }
}

impl Workflow {
    /// Open the workflow
    ///
    /// Find `workflow.toml` in the current working directory or any parent directory. Open the
    /// file, parse it, and return a `Workflow`.
    ///
    /// # Errors
    /// Returns `Err(row::Error)` when the file is not found, cannot be read, or there is a parse
    /// error.
    ///
    pub fn open() -> Result<Self, Error> {
        let (path, file) = find_and_open_workflow()?;
        let mut buffer = BufReader::new(file);
        let mut workflow_string = String::new();
        buffer
            .read_to_string(&mut workflow_string)
            .map_err(|e| Error::FileRead(path.join("workflow.toml"), e))?;

        trace!("Parsing '{}/workflow.toml'.", &path.display());
        Self::open_str(&path, &workflow_string)
    }

    /// Build a workflow from a given path and toml string.
    ///
    /// Parse the contents of the given string as if it were `workflow.toml` at the given `path`.
    ///
    /// # Errors
    /// Returns `Err(row::Error)` when the file is not found, cannot be read, or there is a parse
    /// error.
    ///
    pub(crate) fn open_str(path: &Path, toml: &str) -> Result<Self, Error> {
        let mut workflow: Workflow =
            toml::from_str(toml).map_err(|e| Error::TOMLParse(path.join("workflow.toml"), e))?;
        workflow.root = path.canonicalize()?;
        workflow.validate_and_set_defaults()
    }

    /// Find the action that matches the given name.
    pub fn action_by_name(&self, name: &str) -> Option<&Action> {
        if let Some(action_index) = self.action.iter().position(|a| a.name() == name) {
            Some(&self.action[action_index])
        } else {
            None
        }
    }

    /// Validate a `Workflow` and populate defaults.
    ///
    /// Resolve each action to a fully defined struct with defaults populated
    /// from: The current action, the action named by "from", and the default
    /// action (in that order).
    ///
    fn validate_and_set_defaults(mut self) -> Result<Self, Error> {
        let mut action_names = HashSet::with_capacity(self.action.len());

        if self.default.action.from.is_some() {
            return Err(Error::DefaultActionSetsFrom());
        }

        let source_actions = self.action.clone();

        for (action_idx, action) in self.action.iter_mut().enumerate() {
            if let Some(from) = &action.from {
                if let Some(action_index) = source_actions.iter().position(|a| a.name() == from) {
                    if let Some(recursive_from) = &source_actions[action_index].from {
                        return Err(Error::RecursiveFrom(recursive_from.clone()));
                    }

                    action.resolve(&source_actions[action_index]);
                } else {
                    return Err(Error::FromActionNotFound(from.clone()));
                }
            }

            action.resolve(&self.default.action);

            action_names.insert(action.name().to_string());
            trace!("Validating action '{}'.", action.name());

            if action.name.is_none() {
                return Err(Error::ActionMissingName(action_idx));
            }
            if action.command.is_none() {
                return Err(Error::ActionMissingCommand(action.name().into()));
            }

            // Warn for apparently invalid sort_by.
            for pointer in action.group.sort_by() {
                if !pointer.is_empty() && !pointer.starts_with('/') {
                    warn!("The JSON pointer '{pointer}' does not appear valid. Did you mean '/{pointer}'?");
                }
            }
        }

        for action in &self.action {
            for previous_action in action.previous_actions() {
                if !action_names.contains(previous_action) {
                    return Err(Error::PreviousActionNotFound(
                        previous_action.clone(),
                        action.name().into(),
                    ));
                }
            }

            if let Some(first_action) = self.action_by_name(action.name()) {
                if action.previous_actions != first_action.previous_actions {
                    return Err(Error::DuplicateActionsDifferentPreviousActions(
                        action.name().to_string(),
                    ));
                }
                if action.products != first_action.products {
                    return Err(Error::DuplicateActionsDifferentProducts(
                        action.name().to_string(),
                    ));
                }
            }
        }

        Ok(self)
    }
}

impl Default for Workspace {
    fn default() -> Self {
        Self {
            path: default_workspace_path(),
            value_file: None,
        }
    }
}

/// The default value for workspace.path.
fn default_workspace_path() -> PathBuf {
    PathBuf::from("workspace")
}

/// Parse walltimes from strings.
fn deserialize_duration_from_str<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let duration = Duration::from_str(&s).map_err(serde::de::Error::custom)?;
    Ok(duration)
}

/// Finds and opens the file `workflow.toml`.
///
/// Looks in the current working directory and all parent directories.
///
/// # Errors
/// Returns `Err(row::Error)` when the file is not found or cannot be opened.
///
/// # Returns
/// `Ok(PathBuf, File)` including the path where the file was found and the open file handle.
///
fn find_and_open_workflow() -> Result<(PathBuf, File), Error> {
    let mut path = env::current_dir()?;

    let workflow_file = loop {
        path.push("workflow.toml");
        trace!("Checking {}.", path.display());

        let workflow_file_result = File::open(&path);
        match workflow_file_result {
            Ok(file) => break file,
            Err(error) => match error.kind() {
                io::ErrorKind::NotFound => (),
                _ => return Err(Error::FileRead(path, error)),
            },
        }

        path.pop();
        if !path.pop() {
            return Err(Error::WorkflowNotFound);
        }
    };

    path.pop();
    debug!("Found project in '{}'.", path.display());

    Ok((path, workflow_file))
}

#[cfg(test)]
mod tests {
    use assert_fs::prelude::*;
    use assert_fs::TempDir;
    use serial_test::{parallel, serial};
    use std::env;

    use super::*;

    #[test]
    #[serial]
    fn no_workflow() {
        let temp = TempDir::new().unwrap();
        env::set_current_dir(temp.path()).unwrap();

        let result = find_and_open_workflow();
        assert!(
            result.is_err(),
            "Expected to find no workflow file, but got {result:?}"
        );

        assert!(result
            .unwrap_err()
            .to_string()
            .starts_with("workflow.toml not found in"));
    }

    #[test]
    #[serial]
    fn parent_search() {
        let temp = TempDir::new().unwrap();
        temp.child("workflow.toml").touch().unwrap();

        let sub_path = temp.child("a").child("b").child("c");
        sub_path.create_dir_all().unwrap();
        env::set_current_dir(sub_path.path()).unwrap();

        let result = find_and_open_workflow();

        if let Ok((path, _)) = result {
            assert_eq!(
                path.canonicalize().unwrap(),
                temp.path().canonicalize().unwrap()
            );
        } else {
            panic!("Expected to find a workflow file, but got {result:?}");
        }
    }

    #[test]
    #[parallel]
    fn empty_workflow_file() {
        let temp = TempDir::new().unwrap();
        let workflow = "";
        let workflow = Workflow::open_str(temp.path(), workflow).unwrap();

        assert_eq!(workflow.root, temp.path().canonicalize().unwrap());
        assert_eq!(workflow.workspace.path, PathBuf::from("workspace"));
        assert!(workflow.workspace.value_file.is_none());
        assert_eq!(workflow.default.action, Action::default());
        assert!(workflow.action.is_empty());
    }

    #[test]
    #[parallel]
    fn workspace() {
        let temp = TempDir::new().unwrap();
        let workflow = r#"
[workspace]
path = "p"
value_file = "s"
"#;
        let workflow = Workflow::open_str(temp.path(), workflow).unwrap();

        assert_eq!(workflow.workspace.path, PathBuf::from("p"));
        assert_eq!(workflow.workspace.value_file, Some(PathBuf::from("s")));
    }

    #[test]
    #[parallel]
    fn submit_options_defaults() {
        let temp = TempDir::new().unwrap();
        let workflow = "[default.action.submit_options.a]";
        let workflow = Workflow::open_str(temp.path(), workflow).unwrap();

        assert_eq!(
            workflow.root.canonicalize().unwrap(),
            temp.path().canonicalize().unwrap()
        );

        assert_eq!(workflow.default.action.submit_options.len(), 1);
        assert!(workflow.default.action.submit_options.contains_key("a"));

        let submit_options = workflow.default.action.submit_options.get("a").unwrap();
        assert_eq!(submit_options.account, None);
        assert_eq!(submit_options.setup, None);
        assert!(submit_options.custom.is_empty());
        assert_eq!(submit_options.partition, None);
    }

    #[test]
    #[parallel]
    fn submit_options_nondefault() {
        let temp = TempDir::new().unwrap();
        let workflow = r#"
[default.action.submit_options.a]
account = "my_account"
setup = "module load openmpi"
custom = ["--option1", "--option2"]
partition = "gpu"
"#;
        let workflow = Workflow::open_str(temp.path(), workflow).unwrap();

        assert_eq!(
            workflow.root.canonicalize().unwrap(),
            temp.path().canonicalize().unwrap()
        );

        assert_eq!(workflow.default.action.submit_options.len(), 1);
        assert!(workflow.default.action.submit_options.contains_key("a"));

        let submit_options = workflow.default.action.submit_options.get("a").unwrap();
        assert_eq!(submit_options.account, Some(String::from("my_account")));
        assert_eq!(
            submit_options.setup,
            Some(String::from("module load openmpi"))
        );
        assert_eq!(submit_options.custom, vec!["--option1", "--option2"]);
        assert_eq!(submit_options.partition, Some(String::from("gpu")));
    }

    #[test]
    #[parallel]
    fn action_defaults() {
        let temp = TempDir::new().unwrap();
        let workflow = r#"
[[action]]
name = "b"
command = "c"
"#;
        let workflow = Workflow::open_str(temp.path(), workflow).unwrap();

        assert_eq!(workflow.action.len(), 1);

        let action = workflow.action.first().unwrap();
        assert_eq!(action.name(), "b");
        assert_eq!(action.command(), "c");
        assert!(action.previous_actions.is_none());
        assert!(action.products.is_none());
        assert!(action.launchers.is_none());

        assert_eq!(action.resources.processes, None);
        assert_eq!(action.resources.processes(), Processes::PerSubmission(1));
        assert_eq!(action.resources.threads_per_process, None);
        assert_eq!(action.resources.gpus_per_process, None);
        assert_eq!(action.resources.walltime, None,);
        assert_eq!(
            action.resources.walltime(),
            Walltime::PerDirectory(Duration::new(true, 0, 3600, 0).unwrap())
        );

        assert!(action.submit_options.is_empty());
        assert_eq!(action.group.include, None);
        assert!(action.group.include().is_empty());
        assert_eq!(action.group.sort_by, None);
        assert!(action.group.sort_by().is_empty());
        assert_eq!(action.group.split_by_sort_key, None);
        assert!(!action.group.split_by_sort_key());
        assert_eq!(action.group.maximum_size, None);
        assert_eq!(action.group.submit_whole, None);
        assert!(!action.group.submit_whole());
        assert_eq!(action.group.reverse_sort, None);
        assert!(!action.group.reverse_sort());
    }

    #[test]
    #[parallel]
    fn action_no_name() {
        let temp = TempDir::new().unwrap();
        let workflow = r#"
[[action]]
command = "c"
"#;
        let result = Workflow::open_str(temp.path(), workflow);
        assert!(result.is_err());

        assert!(result.unwrap_err().to_string().contains("missing `name`"));
    }

    #[test]
    #[parallel]
    fn action_no_command() {
        let temp = TempDir::new().unwrap();
        let workflow = r#"
[[action]]
name = "a"
"#;
        let result = Workflow::open_str(temp.path(), workflow);
        assert!(result.is_err());

        assert!(result
            .unwrap_err()
            .to_string()
            .contains("missing `command`"));
    }

    #[test]
    #[parallel]
    fn group_defaults() {
        let temp = TempDir::new().unwrap();
        let workflow = r#"
[[action]]
name = "b"
command = "c"
[action.group]
"#;
        let workflow = Workflow::open_str(temp.path(), workflow).unwrap();

        assert_eq!(workflow.action.len(), 1);

        let action = workflow.action.first().unwrap();

        assert!(action.submit_options.is_empty());
        assert!(action.group.include().is_empty());
        assert!(action.group.sort_by().is_empty());
        assert!(!action.group.split_by_sort_key());
        assert_eq!(action.group.maximum_size, None);
        assert!(!action.group.submit_whole());
        assert!(!action.group.reverse_sort());
    }

    #[test]
    #[parallel]
    fn action_duplicate() {
        let temp = TempDir::new().unwrap();
        let workflow = r#"
[[action]]
name = "b"
command = "c"

[[action]]
name = "b"
command = "d"
"#;
        let result = Workflow::open_str(temp.path(), workflow);
        assert!(result.is_ok());
    }

    #[test]
    #[parallel]
    fn action_duplicate_different_products() {
        let temp = TempDir::new().unwrap();
        let workflow = r#"
[[action]]
name = "b"
command = "c"
products = ["e"]

[[action]]
name = "b"
command = "d"
products = ["b"]
"#;
        let result = Workflow::open_str(temp.path(), workflow);
        assert!(matches!(
            result,
            Err(Error::DuplicateActionsDifferentProducts(_))
        ));

        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must have the same `products`"));
    }

    #[test]
    #[parallel]
    fn action_duplicate_different_previous_actions() {
        let temp = TempDir::new().unwrap();
        let workflow = r#"
[[action]]
name = "b"
command = "c"

[[action]]
name = "b"
command = "d"
previous_actions = ["a"]

[[action]]
name = "a"
command = "e"
"#;
        let result = Workflow::open_str(temp.path(), workflow);
        assert!(matches!(
            result,
            Err(Error::DuplicateActionsDifferentPreviousActions(_))
        ));

        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must have the same `previous_actions`"));
    }

    #[test]
    #[parallel]
    fn action_launchers() {
        let temp = TempDir::new().unwrap();
        let workflow = r#"
[[action]]
name = "b"
command = "c"
launchers = ["openmp", "mpi"]
"#;

        let workflow = Workflow::open_str(temp.path(), workflow).unwrap();

        assert_eq!(workflow.action.len(), 1);

        let action = workflow.action.first().unwrap();
        assert_eq!(
            action.launchers(),
            vec!["openmp".to_string(), "mpi".to_string()]
        );
    }

    #[test]
    #[parallel]
    fn action_previous_actions() {
        let temp = TempDir::new().unwrap();
        let workflow = r#"
[[action]]
name = "b"
command = "c"

[[action]]
name = "d"
command = "e"
previous_actions = ["b"]
"#;

        let workflow = Workflow::open_str(temp.path(), workflow).unwrap();

        assert_eq!(workflow.action.len(), 2);

        let action = workflow.action.get(1).unwrap();
        assert_eq!(action.previous_actions(), vec!["b".to_string()]);

        let action_a = workflow.action_by_name("b");
        assert_eq!(action_a.unwrap().command(), "c");

        let action_d = workflow.action_by_name("d");
        assert_eq!(action_d.unwrap().command(), "e");

        assert!(workflow.action_by_name("f").is_none());
    }

    #[test]
    #[parallel]
    fn previous_action_error() {
        let temp = TempDir::new().unwrap();
        let workflow = r#"
[[action]]
name = "b"
command = "c"
previous_actions = ["a"]
"#;
        let result = Workflow::open_str(temp.path(), workflow);
        assert!(
            result.is_err(),
            "Expected previous action error, but got {result:?}"
        );

        assert!(result
            .unwrap_err()
            .to_string()
            .starts_with("Previous action 'a' not found"));
    }

    #[test]
    #[parallel]
    fn action_resources() {
        let temp = TempDir::new().unwrap();
        let workflow = r#"
[[action]]
name = "b"
command = "c"
[action.resources]
processes.per_submission = 12
threads_per_process = 8
gpus_per_process = 1
walltime.per_submission = "4d, 05:32:11"
"#;

        let workflow = Workflow::open_str(temp.path(), workflow).unwrap();

        assert_eq!(workflow.action.len(), 1);

        let action = workflow.action.first().unwrap();
        assert_eq!(action.resources.processes(), Processes::PerSubmission(12));
        assert_eq!(action.resources.threads_per_process, Some(8));
        assert_eq!(action.resources.gpus_per_process, Some(1));
        assert_eq!(
            action.resources.walltime(),
            Walltime::PerSubmission(
                Duration::new(true, 4, 5 * 3600 + 32 * 60 + 11, 0)
                    .expect("this should be a valid Duration"),
            )
        );
    }

    #[test]
    #[parallel]
    fn action_resources_per_directory() {
        let temp = TempDir::new().unwrap();
        let workflow = r#"
[[action]]
name = "b"
command = "c"
[action.resources]
processes.per_directory = 1
walltime.per_directory = "00:01"
"#;

        let workflow = Workflow::open_str(temp.path(), workflow).unwrap();

        assert_eq!(workflow.action.len(), 1);

        let action = workflow.action.first().unwrap();
        assert_eq!(action.resources.processes(), Processes::PerDirectory(1));

        assert_eq!(
            action.resources.walltime(),
            Walltime::PerDirectory(
                Duration::new(true, 0, 60, 0).expect("this should be a valid Duration")
            )
        );
    }

    #[test]
    #[parallel]
    fn processes_duplicate() {
        let temp = TempDir::new().unwrap();
        let workflow = r#"
[[action]]
name = "b"
command = "c"
[action.resources]
processes.per_submission = 1
processes.per_directory = 2
"#;
        let result = Workflow::open_str(temp.path(), workflow);
        assert!(
            matches!(result, Err(Error::TOMLParse(..))),
            "Expected duplicate processes error, but got {result:?}"
        );

        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("wanted exactly 1 element"),
            "Expected 'wanted exactly 1 element', got {err:?}"
        );
    }

    #[test]
    #[parallel]
    fn walltime_duplicate() {
        let temp = TempDir::new().unwrap();
        let workflow = r#"
[[action]]
name = "b"
command = "c"
[action.resources]
walltime.per_submission = "00:01"
walltime.per_directory = "01:00"
"#;
        let result = Workflow::open_str(temp.path(), workflow);
        assert!(
            matches!(result, Err(Error::TOMLParse(..))),
            "Expected duplicate walltime error, but got {result:?}"
        );

        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("wanted exactly 1 element"),
            "Expected 'wanted exactly 1 element', got {err:?}"
        );
    }
    #[test]
    #[parallel]
    fn action_products() {
        let temp = TempDir::new().unwrap();
        let workflow = r#"
[[action]]
name = "b"
command = "c"
products = ["d", "e"]
"#;

        let workflow = Workflow::open_str(temp.path(), workflow).unwrap();

        assert_eq!(workflow.action.len(), 1);

        let action = workflow.action.first().unwrap();
        assert_eq!(action.products(), vec!["d".to_string(), "e".to_string()]);
    }

    #[test]
    #[parallel]
    fn action_group() {
        let temp = TempDir::new().unwrap();
        let workflow = r#"
[[action]]
name = "b"
command = "c"
[action.group]
include = [["/d", "==", 5], ["/float", ">", 6.5], ["/string", "<", "str"], ["/array", "==", [1,2,3]], ["/bool", "==", false]]
sort_by = ["/sort"]
split_by_sort_key = true
maximum_size = 10
submit_whole = true
reverse_sort = true
"#;

        let workflow = Workflow::open_str(temp.path(), workflow).unwrap();

        assert_eq!(workflow.action.len(), 1);

        let action = workflow.action.first().unwrap();
        assert_eq!(
            action.group.include(),
            vec![
                (
                    "/d".to_string(),
                    Comparison::EqualTo,
                    serde_json::Value::from(5)
                ),
                (
                    "/float".to_string(),
                    Comparison::GreaterThan,
                    serde_json::Value::from(6.5)
                ),
                (
                    "/string".to_string(),
                    Comparison::LessThan,
                    serde_json::Value::from("str")
                ),
                (
                    "/array".to_string(),
                    Comparison::EqualTo,
                    serde_json::Value::from(vec![1, 2, 3])
                ),
                (
                    "/bool".to_string(),
                    Comparison::EqualTo,
                    serde_json::Value::from(false)
                )
            ]
        );
        assert_eq!(action.group.sort_by(), vec![String::from("/sort")]);
        assert!(action.group.split_by_sort_key());
        assert_eq!(action.group.maximum_size, Some(10));
        assert!(action.group.submit_whole());
        assert!(action.group.reverse_sort());
    }

    #[test]
    #[parallel]
    fn action_submit_options_none() {
        let temp = TempDir::new().unwrap();
        let workflow = r#"
[[action]]
name = "b"
command = "c"
"#;

        let workflow = Workflow::open_str(temp.path(), workflow).unwrap();

        assert_eq!(workflow.action.len(), 1);

        let action = workflow.action.first().unwrap();
        assert!(action.submit_options.is_empty());
    }

    #[test]
    #[parallel]
    fn action_submit_options_default() {
        let temp = TempDir::new().unwrap();
        let workflow = r#"
[[action]]
name = "b"
command = "c"

[action.submit_options.d]
"#;

        let workflow = Workflow::open_str(temp.path(), workflow).unwrap();

        assert_eq!(workflow.action.len(), 1);

        let action = workflow.action.first().unwrap();
        assert!(!action.submit_options.is_empty());
        assert!(action.submit_options.contains_key("d"));

        let submit_options = action.submit_options.get("d").unwrap();
        assert_eq!(submit_options.account, None);
        assert_eq!(submit_options.setup, None);
        assert!(submit_options.custom.is_empty());
        assert_eq!(submit_options.partition, None);
    }

    #[test]
    #[parallel]
    fn action_submit_options_nondefault() {
        let temp = TempDir::new().unwrap();
        let workflow = r#"
[[action]]
name = "b"
command = "c"

[action.submit_options.d]
account = "e"
setup = "f"
custom = ["g", "h"]
partition = "i"
"#;

        let workflow = Workflow::open_str(temp.path(), workflow).unwrap();

        assert_eq!(workflow.action.len(), 1);

        let action = workflow.action.first().unwrap();
        assert!(!action.submit_options.is_empty());
        assert!(action.submit_options.contains_key("d"));

        let submit_options = action.submit_options.get("d").unwrap();
        assert_eq!(submit_options.account, Some("e".to_string()));
        assert_eq!(submit_options.setup, Some("f".to_string()));
        assert_eq!(submit_options.custom, vec!["g", "h"]);
        assert_eq!(submit_options.partition, Some("i".to_string()));
    }

    #[test]
    #[parallel]
    fn action_submit_options_global() {
        let temp = TempDir::new().unwrap();
        let workflow = r#"
[default.action.submit_options.d]
account = "e"
setup = "f"
custom = ["g", "h"]
partition = "i"

[[action]]
name = "b"
command = "c"
"#;

        let workflow = Workflow::open_str(temp.path(), workflow).unwrap();

        assert_eq!(workflow.action.len(), 1);

        let action = workflow.action.first().unwrap();
        assert!(!action.submit_options.is_empty());
        assert!(action.submit_options.contains_key("d"));

        let submit_options = action.submit_options.get("d").unwrap();
        assert_eq!(submit_options.account, Some("e".to_string()));
        assert_eq!(submit_options.setup, Some("f".to_string()));
        assert_eq!(submit_options.custom, vec!["g", "h"]);
        assert_eq!(submit_options.partition, Some("i".to_string()));
    }

    #[test]
    #[parallel]
    fn action_submit_options_no_override() {
        let temp = TempDir::new().unwrap();
        let workflow = r#"
[default.action.submit_options.d]
account = "e"
setup = "f"
custom = ["g", "h"]
partition = "i"

[[action]]
name = "b"
command = "c"

[action.submit_options.d]
account = "j"
setup = "k"
custom = ["l", "m"]
partition = "n"
"#;

        let workflow = Workflow::open_str(temp.path(), workflow).unwrap();

        assert_eq!(workflow.action.len(), 1);

        let action = workflow.action.first().unwrap();
        assert!(!action.submit_options.is_empty());
        assert!(action.submit_options.contains_key("d"));

        let submit_options = action.submit_options.get("d").unwrap();
        assert_eq!(submit_options.account, Some("j".to_string()));
        assert_eq!(submit_options.setup, Some("k".to_string()));
        assert_eq!(submit_options.custom, vec!["l", "m"]);
        assert_eq!(submit_options.partition, Some("n".to_string()));
    }

    #[test]
    #[parallel]
    fn action_submit_options_override() {
        let temp = TempDir::new().unwrap();
        let workflow = r#"
[default.action.submit_options.d]
account = "e"
setup = "f"
custom = ["g", "h"]
partition = "i"

[[action]]
name = "b"
command = "c"

[action.submit_options.d]
"#;

        let workflow = Workflow::open_str(temp.path(), workflow).unwrap();

        assert_eq!(workflow.action.len(), 1);

        let action = workflow.action.first().unwrap();
        assert!(!action.submit_options.is_empty());
        assert!(action.submit_options.contains_key("d"));

        let submit_options = action.submit_options.get("d").unwrap();
        assert_eq!(submit_options.account, Some("e".to_string()));
        assert_eq!(submit_options.setup, Some("f".to_string()));
        assert_eq!(submit_options.custom, vec!["g", "h"]);
        assert_eq!(submit_options.partition, Some("i".to_string()));
    }

    #[test]
    #[parallel]
    fn default_action_from() {
        let temp = TempDir::new().unwrap();
        let workflow = r#"
[default.action]
from = "a"
"#;
        let result = Workflow::open_str(temp.path(), workflow);
        assert!(result.is_err());

        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must not set `from`"));
    }

    #[test]
    #[parallel]
    fn empty_action_default() {
        let temp = TempDir::new().unwrap();
        let workflow = "
[default.action]
";

        let workflow = Workflow::open_str(temp.path(), workflow).unwrap();

        assert_eq!(workflow.action.len(), 0);

        let action = workflow.default.action;
        assert_eq!(action.name, None);
        assert_eq!(action.command, None);
        assert_eq!(action.launchers, None);
        assert_eq!(action.previous_actions, None);
        assert_eq!(action.products, None);
        assert_eq!(action.resources.processes, None);
        assert_eq!(action.resources.threads_per_process, None);
        assert_eq!(action.resources.gpus_per_process, None);
        assert_eq!(action.resources.walltime, None);
        assert!(action.submit_options.is_empty());
        assert_eq!(action.group.include, None);
        assert_eq!(action.group.sort_by, None);
        assert_eq!(action.group.split_by_sort_key, None);
        assert_eq!(action.group.reverse_sort, None);
        assert_eq!(action.group.maximum_size, None);
        assert_eq!(action.group.submit_whole, None);
        assert_eq!(action.from, None);
    }

    #[test]
    #[parallel]
    fn action_default() {
        let temp = TempDir::new().unwrap();
        let workflow = r#"
[default.action]
name = "a"
command = "b"
launchers = ["c"]
previous_actions = ["d"]
products = ["e"]

[default.action.resources]
processes.per_directory = 2
threads_per_process = 3
gpus_per_process = 4
walltime.per_submission = "00:00:01"

# submit_options is tested above

[default.action.group]
include = [["/f", "==", 5]]
sort_by = ["/g"]
split_by_sort_key = true
reverse_sort = true
maximum_size = 6
submit_whole = true

[[action]]

[[action]]
name = "d"
"#;

        let workflow = Workflow::open_str(temp.path(), workflow).unwrap();

        assert_eq!(workflow.action.len(), 2);

        let action = workflow.action.first().unwrap();
        assert_eq!(action.name(), "a");
        assert_eq!(action.command(), "b");
        assert_eq!(action.launchers(), vec!["c"]);
        assert_eq!(action.previous_actions(), vec!["d"]);
        assert_eq!(action.products(), vec!["e"]);
        assert_eq!(action.resources.processes(), Processes::PerDirectory(2));
        assert_eq!(action.resources.threads_per_process, Some(3));
        assert_eq!(action.resources.gpus_per_process, Some(4));
        assert_eq!(
            action.resources.walltime(),
            Walltime::PerSubmission(Duration::new(true, 0, 1, 0).unwrap())
        );
        assert!(action.submit_options.is_empty());
        assert_eq!(
            action.group.include(),
            vec![("/f".into(), Comparison::EqualTo, serde_json::Value::from(5))]
        );
        assert_eq!(action.group.sort_by(), vec!["/g"]);
        assert!(action.group.split_by_sort_key());
        assert!(action.group.reverse_sort());
        assert_eq!(action.group.maximum_size, Some(6));
        assert!(action.group.submit_whole());
        assert_eq!(action.from, None);
    }

    #[test]
    #[parallel]
    fn action_override_default() {
        let temp = TempDir::new().unwrap();
        let workflow = r#"
[default.action]
name = "a"
command = "b"
launchers = ["c"]
products = ["e"]

[default.action.resources]
processes.per_directory = 2
threads_per_process = 3
gpus_per_process = 4
walltime.per_submission = "00:00:01"

# submit_options is tested above

[default.action.group]
include = [["/f", "==", 5]]
sort_by = ["/g"]
split_by_sort_key = true
reverse_sort = true
maximum_size = 6
submit_whole = true

[[action]]
name = "aa"
command = "bb"
launchers = ["cc"]
previous_actions = ["dd"]
products = ["ee"]

[action.resources]
processes.per_directory = 4
threads_per_process = 6
gpus_per_process = 8
walltime.per_submission = "00:00:02"

# submit_options is tested above

[action.group]
include = [["/ff", "==", 10]]
sort_by = ["/gg"]
split_by_sort_key = false
reverse_sort = false
maximum_size = 12
submit_whole = false

[[action]]
name = "dd"
"#;

        let workflow = Workflow::open_str(temp.path(), workflow).unwrap();

        assert_eq!(workflow.action.len(), 2);

        let action = workflow.action.first().unwrap();
        assert_eq!(action.name(), "aa");
        assert_eq!(action.command(), "bb");
        assert_eq!(action.launchers(), vec!["cc"]);
        assert_eq!(action.previous_actions(), vec!["dd"]);
        assert_eq!(action.products(), vec!["ee"]);
        assert_eq!(action.resources.processes(), Processes::PerDirectory(4));
        assert_eq!(action.resources.threads_per_process, Some(6));
        assert_eq!(action.resources.gpus_per_process, Some(8));
        assert_eq!(
            action.resources.walltime(),
            Walltime::PerSubmission(Duration::new(true, 0, 2, 0).unwrap())
        );
        assert!(action.submit_options.is_empty());
        assert_eq!(
            action.group.include(),
            vec![(
                "/ff".into(),
                Comparison::EqualTo,
                serde_json::Value::from(10)
            )]
        );
        assert_eq!(action.group.sort_by(), vec!["/gg"]);
        assert!(!action.group.split_by_sort_key());
        assert!(!action.group.reverse_sort());
        assert_eq!(action.group.maximum_size, Some(12));
        assert!(!action.group.submit_whole());
        assert_eq!(action.from, None);
    }

    #[test]
    #[parallel]
    fn action_from() {
        let temp = TempDir::new().unwrap();
        let workflow = r#"
[[action]]
name = "a"
command = "b"
launchers = ["c"]
previous_actions = ["d"]
products = ["e"]

[default.action.resources]
processes.per_directory = 2
threads_per_process = 3
gpus_per_process = 4
walltime.per_submission = "00:00:01"

# submit_options is tested above

[default.action.group]
include = [["/f", "==", 5]]
sort_by = ["/g"]
split_by_sort_key = true
reverse_sort = true
maximum_size = 6
submit_whole = true

[[action]]
from = "a"

[[action]]
name = "d"
command = "e"
"#;

        let workflow = Workflow::open_str(temp.path(), workflow).unwrap();

        assert_eq!(workflow.action.len(), 3);

        let action = &workflow.action[1];
        assert_eq!(action.name(), "a");
        assert_eq!(action.command(), "b");
        assert_eq!(action.launchers(), vec!["c"]);
        assert_eq!(action.previous_actions(), vec!["d"]);
        assert_eq!(action.products(), vec!["e"]);
        assert_eq!(action.resources.processes(), Processes::PerDirectory(2));
        assert_eq!(action.resources.threads_per_process, Some(3));
        assert_eq!(action.resources.gpus_per_process, Some(4));
        assert_eq!(
            action.resources.walltime(),
            Walltime::PerSubmission(Duration::new(true, 0, 1, 0).unwrap())
        );
        assert!(action.submit_options.is_empty());
        assert_eq!(
            action.group.include(),
            vec![("/f".into(), Comparison::EqualTo, serde_json::Value::from(5))]
        );
        assert_eq!(action.group.sort_by(), vec!["/g"]);
        assert!(action.group.split_by_sort_key());
        assert!(action.group.reverse_sort());
        assert_eq!(action.group.maximum_size, Some(6));
        assert!(action.group.submit_whole());
        assert_eq!(action.from, Some("a".into()));
    }

    #[test]
    #[parallel]
    fn action_override_from() {
        let temp = TempDir::new().unwrap();
        let workflow = r#"
[[action]]
name = "a"
command = "b"
launchers = ["c"]
previous_actions = ["d"]
products = ["e"]

[default.action.resources]
processes.per_directory = 2
threads_per_process = 3
gpus_per_process = 4
walltime.per_submission = "00:00:01"

# submit_options is tested above

[default.action.group]
include = [["/f", "==", 5]]
sort_by = ["/g"]
split_by_sort_key = true
reverse_sort = true
maximum_size = 6
submit_whole = true

[[action]]
from = "a"

name = "aa"
command = "bb"
launchers = ["cc"]
previous_actions = ["dd"]
products = ["ee"]

[action.resources]
processes.per_directory = 4
threads_per_process = 6
gpus_per_process = 8
walltime.per_submission = "00:00:02"

# submit_options is tested above

[action.group]
include = [["/ff", "==", 10]]
sort_by = ["/gg"]
split_by_sort_key = false
reverse_sort = false
maximum_size = 12
submit_whole = false

[[action]]
name = "dd"
command = "ee"

[[action]]
name = "d"
command = "e"
"#;

        let workflow = Workflow::open_str(temp.path(), workflow).unwrap();

        assert_eq!(workflow.action.len(), 4);

        let action = &workflow.action[1];
        assert_eq!(action.name(), "aa");
        assert_eq!(action.command(), "bb");
        assert_eq!(action.launchers(), vec!["cc"]);
        assert_eq!(action.previous_actions(), vec!["dd"]);
        assert_eq!(action.products(), vec!["ee"]);
        assert_eq!(action.resources.processes(), Processes::PerDirectory(4));
        assert_eq!(action.resources.threads_per_process, Some(6));
        assert_eq!(action.resources.gpus_per_process, Some(8));
        assert_eq!(
            action.resources.walltime(),
            Walltime::PerSubmission(Duration::new(true, 0, 2, 0).unwrap())
        );
        assert!(action.submit_options.is_empty());
        assert_eq!(
            action.group.include(),
            vec![(
                "/ff".into(),
                Comparison::EqualTo,
                serde_json::Value::from(10)
            )]
        );
        assert_eq!(action.group.sort_by(), vec!["/gg"]);
        assert!(!action.group.split_by_sort_key());
        assert!(!action.group.reverse_sort());
        assert_eq!(action.group.maximum_size, Some(12));
        assert!(!action.group.submit_whole());
        assert_eq!(action.from, Some("a".into()));
    }

    #[test]
    #[parallel]
    fn action_override_mixed() {
        let temp = TempDir::new().unwrap();
        let workflow = r#"
[default.action]
resources.threads_per_process = 2

[[action]]
name = "a"
command = "b"
resources.gpus_per_process = 4

[[action]]
from = "a"
resources.processes.per_directory = 8
"#;

        let workflow = Workflow::open_str(temp.path(), workflow).unwrap();

        assert_eq!(workflow.action.len(), 2);

        let action = &workflow.action[1];
        assert_eq!(action.name(), "a");
        assert_eq!(action.command(), "b");
        assert_eq!(action.resources.processes(), Processes::PerDirectory(8));
        assert_eq!(action.resources.threads_per_process, Some(2));
        assert_eq!(action.resources.gpus_per_process, Some(4));
    }

    #[test]
    #[parallel]
    fn total_processes() {
        let r = Resources {
            processes: Some(Processes::PerSubmission(10)),
            ..Resources::default()
        };

        assert_eq!(r.total_processes(10), 10);
        assert_eq!(r.total_processes(100), 10);
        assert_eq!(r.total_processes(1000), 10);

        let r = Resources {
            processes: Some(Processes::PerDirectory(10)),
            ..Resources::default()
        };

        assert_eq!(r.total_processes(10), 100);
        assert_eq!(r.total_processes(100), 1000);
        assert_eq!(r.total_processes(1000), 10000);
    }

    #[test]
    #[parallel]
    fn total_cpus() {
        let r = Resources {
            processes: Some(Processes::PerSubmission(10)),
            threads_per_process: Some(2),
            ..Resources::default()
        };

        assert_eq!(r.total_cpus(10), 20);
        assert_eq!(r.total_cpus(100), 20);
        assert_eq!(r.total_cpus(1000), 20);

        let r = Resources {
            processes: Some(Processes::PerDirectory(10)),
            threads_per_process: None,
            ..Resources::default()
        };

        assert_eq!(r.total_cpus(10), 100);
        assert_eq!(r.total_cpus(100), 1000);
        assert_eq!(r.total_cpus(1000), 10000);
    }

    #[test]
    #[parallel]
    fn total_gpus() {
        let r = Resources {
            processes: Some(Processes::PerSubmission(10)),
            gpus_per_process: Some(2),
            ..Resources::default()
        };

        assert_eq!(r.total_gpus(10), 20);
        assert_eq!(r.total_gpus(100), 20);
        assert_eq!(r.total_gpus(1000), 20);

        let r = Resources {
            processes: Some(Processes::PerDirectory(10)),
            gpus_per_process: None,
            ..Resources::default()
        };

        assert_eq!(r.total_gpus(10), 0);
        assert_eq!(r.total_gpus(100), 0);
        assert_eq!(r.total_gpus(1000), 0);
    }

    #[test]
    #[parallel]
    fn total_walltime() {
        let r = Resources {
            walltime: Some(Walltime::PerDirectory(
                Duration::new(true, 1, 3600, 0).unwrap(),
            )),
            ..Resources::default()
        };

        assert_eq!(
            r.total_walltime(2),
            Duration::new(true, 2, 2 * 3600, 0).unwrap()
        );
        assert_eq!(
            r.total_walltime(4),
            Duration::new(true, 4, 4 * 3600, 0).unwrap()
        );
        assert_eq!(
            r.total_walltime(8),
            Duration::new(true, 8, 8 * 3600, 0).unwrap()
        );

        let r = Resources {
            walltime: Some(Walltime::PerSubmission(
                Duration::new(true, 1, 3600, 0).unwrap(),
            )),
            ..Resources::default()
        };

        assert_eq!(
            r.total_walltime(2),
            Duration::new(true, 1, 3600, 0).unwrap()
        );
        assert_eq!(
            r.total_walltime(4),
            Duration::new(true, 1, 3600, 0).unwrap()
        );
        assert_eq!(
            r.total_walltime(8),
            Duration::new(true, 1, 3600, 0).unwrap()
        );
    }

    #[test]
    #[parallel]
    fn resource_cost() {
        let r = Resources {
            processes: Some(Processes::PerSubmission(10)),
            walltime: Some(Walltime::PerDirectory(
                Duration::new(true, 0, 3600, 0).unwrap(),
            )),
            ..Resources::default()
        };

        assert_eq!(r.cost(1), ResourceCost::with_values(10.0, 0.0));
        assert_eq!(r.cost(2), ResourceCost::with_values(20.0, 0.0));
        assert_eq!(r.cost(4), ResourceCost::with_values(40.0, 0.0));

        let r = Resources {
            processes: Some(Processes::PerSubmission(10)),
            walltime: Some(Walltime::PerDirectory(
                Duration::new(true, 0, 3600, 0).unwrap(),
            )),
            threads_per_process: Some(4),
            ..Resources::default()
        };

        assert_eq!(r.cost(1), ResourceCost::with_values(40.0, 0.0));
        assert_eq!(r.cost(2), ResourceCost::with_values(80.0, 0.0));
        assert_eq!(r.cost(4), ResourceCost::with_values(160.0, 0.0));

        let r = Resources {
            processes: Some(Processes::PerSubmission(10)),
            walltime: Some(Walltime::PerDirectory(
                Duration::new(true, 0, 3600, 0).unwrap(),
            )),
            threads_per_process: Some(4),
            gpus_per_process: Some(2),
        };

        assert_eq!(r.cost(1), ResourceCost::with_values(0.0, 20.0));
        assert_eq!(r.cost(2), ResourceCost::with_values(0.0, 40.0));
        assert_eq!(r.cost(4), ResourceCost::with_values(0.0, 80.0));
    }
}
