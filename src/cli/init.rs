// Copyright (c) 2024 The Regents of the University of Michigan.
// Part of row, released under the BSD 3-Clause License.

use clap::Args;
use log::{debug, info, trace, warn};
use path_absolutize::Absolutize;
use std::fmt::Write as _;
use std::fs;
use std::io::Write;
use std::path::{self, Path, PathBuf};

use crate::cli::GlobalOptions;
use row::{Error, DATA_DIRECTORY_NAME};

#[derive(Args, Debug)]
pub struct Arguments {
    /// Configure `workflow.toml` for signac.
    #[arg(long, group = "workspace_group", display_order = 0)]
    signac: bool,

    /// The workspace directory name.
    #[arg(short, long, default_value_t=String::from("workspace"), group="workspace_group", display_order = 0)]
    workspace: String,

    /// Directory to initialize.
    #[arg(display_order = 0)]
    directory: PathBuf,
}

fn is_project(path: &Path) -> Result<(bool, PathBuf), Error> {
    let mut path = PathBuf::from(path);

    let found = loop {
        path.push("workflow.toml");
        trace!("Checking {}.", path.display());

        if path
            .try_exists()
            .map_err(|e| Error::DirectoryRead(path.clone(), e))?
        {
            break true;
        }

        path.pop();
        if !path.pop() {
            break false;
        }
    };

    path.pop();

    Ok((found, path))
}

/// Initialize a new row project directory.
pub fn init<W: Write>(
    _options: &GlobalOptions,
    args: &Arguments,
    output: &mut W,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("Scanning the workspace for completed actions.");

    if args.workspace.contains(path::MAIN_SEPARATOR_STR) {
        return Err(Box::new(row::Error::WorkspacePathNotRelative(
            args.workspace.clone(),
        )));
    }

    let project_directory = args.directory.absolutize()?;
    let (project_found, existing_path) = is_project(&project_directory)?;

    match (project_found, existing_path == project_directory) {
        (true, true) => return Err(Box::new(row::Error::ProjectExists(existing_path))),
        (true, false) => return Err(Box::new(row::Error::ParentProjectExists(existing_path))),
        (_, _) => (),
    }

    if project_directory.clone().join(DATA_DIRECTORY_NAME).exists() {
        return Err(Box::new(row::Error::ProjectCacheExists(
            project_directory.into(),
        )));
    }

    if project_directory.exists() {
        warn!("'{}' already exists.", project_directory.display());
    }

    let workspace_directory = project_directory.clone().join(&args.workspace);
    info!("Creating directory '{}'", workspace_directory.display());
    fs::create_dir_all(&workspace_directory)
        .map_err(|e| Error::DirectoryCreate(workspace_directory.clone(), e))?;

    let mut workflow = String::new();
    if args.signac {
        let _ = writeln!(
            workflow,
            r#"[workspace]
value_file = "signac_statepoint.json""#
        );
    }

    if args.workspace != "workspace" {
        let _ = writeln!(
            workflow,
            r#"[workspace]
path = '{}'"#,
            args.workspace
        );
    }

    let workflow_path = project_directory.clone().join("workflow.toml");
    info!("Creating file '{}'", workflow_path.display());
    fs::write(&workflow_path, &workflow).map_err(|e| Error::FileWrite(workflow_path, e))?;

    writeln!(
        output,
        "Created row project in '{}'",
        project_directory.display()
    )?;

    Ok(())
}
