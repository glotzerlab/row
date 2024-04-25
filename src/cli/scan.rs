use clap::Args;
use log::{debug, info, trace, warn};
use postcard;
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::PathBuf;
use uuid::Uuid;

use crate::cli::{self, GlobalOptions};
use row::workflow::Workflow;
use row::{
    workspace, Error, MultiProgressContainer, COMPLETED_DIRECTORY_NAME, DATA_DIRECTORY_NAME,
};

#[derive(Args, Debug)]
pub struct ScanArgs {
    /// Select the action to scan (defaults to all).
    #[arg(short, long, display_order = 0)]
    action: Option<String>,

    /// Select directories to scan (defaults to all). Use 'scan -' to read from stdin.
    directories: Vec<PathBuf>,
}

/// Scan directories and determine whether a given action (or all actions) have completed.
///
/// Write the resulting list of completed directories to a completion pack file.
///
pub fn scan(
    options: GlobalOptions,
    args: ScanArgs,
    multi_progress: &mut MultiProgressContainer,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("Scanning the workspace for completed actions.");

    let workflow = Workflow::open()?;

    let query_directories = cli::parse_directories(args.directories, || {
        workspace::list_directories(&workflow, multi_progress)
    })?;

    let mut complete = workspace::find_completed_directories(
        &workflow,
        query_directories,
        options.io_threads,
        multi_progress,
    )
    .get()?;

    let mut matching_action_count = 0;
    for action in workflow.action {
        if let Some(selection) = args.action.as_ref() {
            if selection != &action.name {
                complete.remove(&action.name);
                continue;
            }
        }
        trace!(
            "Including complete directories for action '{}'.",
            action.name
        );

        matching_action_count += 1;
    }

    if matching_action_count == 0 {
        warn!("No actions scanned.");
        return Ok(());
    }

    if complete.is_empty() {
        info!("Found no completed actions.");
        return Ok(());
    }

    debug!("Serializing completed actions.");
    let bytes = postcard::to_stdvec(&complete)
        .map_err(|e| Error::PostcardSerialize("completed".into(), e))?;

    let id = Uuid::new_v4();

    let complete_directory = workflow
        .root
        .join(DATA_DIRECTORY_NAME)
        .join(COMPLETED_DIRECTORY_NAME);
    let filename = complete_directory
        .join(id.simple().to_string())
        .with_extension("postcard");
    let tmp_filename = filename.with_extension("tmp");

    fs::create_dir_all(&complete_directory)
        .map_err(|e| Error::DirectoryCreate(complete_directory, e))?;

    trace!(
        "Writing {} bytes to '{}'.",
        bytes.len(),
        tmp_filename.display().to_string()
    );
    let mut file =
        File::create_new(&tmp_filename).map_err(|e| Error::FileWrite(tmp_filename.clone(), e))?;
    file.write_all(&bytes)
        .map_err(|e| Error::FileWrite(tmp_filename.clone(), e))?;
    file.sync_all()
        .map_err(|e| Error::FileWrite(tmp_filename.clone(), e))?;
    drop(file);

    fs::rename(&tmp_filename, &filename).map_err(|e| Error::FileWrite(filename, e))?;

    for (action, completed_directories) in complete {
        let word = if completed_directories.len() == 1 {
            "directory"
        } else {
            "directories"
        };
        info!(
            "Found {} completed {word} for action '{action}'.",
            completed_directories.len()
        );
    }

    Ok(())
}
