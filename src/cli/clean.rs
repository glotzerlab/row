// Copyright (c) 2024 The Regents of the University of Michigan.
// Part of row, released under the BSD 3-Clause License.

use clap::Args;
use log::{debug, info, warn};
use std::error::Error;
use std::{fs, io};

use crate::cli::GlobalOptions;
use row::project::Project;
use row::MultiProgressContainer;
use row::{
    COMPLETED_CACHE_FILE_NAME, DATA_DIRECTORY_NAME, DIRECTORY_CACHE_FILE_NAME,
    SUBMITTED_CACHE_FILE_NAME,
};

#[derive(Args, Debug)]
pub struct Arguments {
    #[command(flatten)]
    selection: Option<Selection>,

    /// Force removal of the completed and/or submitted cache when there are submitted jobs.
    #[arg(long, display_order = 0)]
    force: bool,
}

#[derive(Args, Debug)]
#[group(multiple = true)]
#[allow(clippy::struct_excessive_bools)]
pub struct Selection {
    /// Remove the directory cache.
    #[arg(long, display_order = 0)]
    directory: bool,

    /// Remove the submitted cache.
    #[arg(long, display_order = 0)]
    submitted: bool,

    /// Remove the completed cache.
    #[arg(long, display_order = 0)]
    completed: bool,
}

/// Remove row cache files.
pub fn clean(
    options: &GlobalOptions,
    args: &Arguments,
    multi_progress: &mut MultiProgressContainer,
) -> Result<(), Box<dyn Error>> {
    debug!("Cleaning cache files.");
    let mut project = Project::open(options.io_threads, &options.cluster, multi_progress)?;

    // Delete all existing completion staging files.
    project.close(multi_progress)?;

    let selection = args.selection.as_ref().unwrap_or(&Selection {directory: true, submitted: true, completed: true});

    let num_submitted = project.state().num_submitted();
    if num_submitted > 0 {
        let force_needed = selection.completed || selection.submitted;

        if force_needed {
            warn!("There are {num_submitted} directories with submitted jobs.");
        }
        if selection.submitted {
            warn!("The submitted cache is not recoverable. Row may resubmit running jobs.");
        }
        if selection.completed {
            warn!("These jobs may add to the completed cache after it is cleaned.");
        }
        if force_needed && !args.force {
            warn!("You should wait for these jobs to complete.");
            return Err(Box::new(row::Error::ForceCleanNeeded));
        }
    }

    let data_directory = project.workflow().root.join(DATA_DIRECTORY_NAME);

    if selection.submitted {
        let path = data_directory.join(SUBMITTED_CACHE_FILE_NAME);
        info!("Removing '{}'.", path.display());
        if let Err(error) = fs::remove_file(&path) {
            match error.kind() {
                io::ErrorKind::NotFound => (),
                _ => return Err(Box::new(row::Error::FileRemove(path.clone(), error))),
            }
        }
    }
    if selection.completed {
        let path = data_directory.join(COMPLETED_CACHE_FILE_NAME);
        info!("Removing '{}'.", path.display());
        if let Err(error) = fs::remove_file(&path) {
            match error.kind() {
                io::ErrorKind::NotFound => (),
                _ => return Err(Box::new(row::Error::FileRemove(path.clone(), error))),
            }
        }
    }
    if selection.directory {
        let path = data_directory.join(DIRECTORY_CACHE_FILE_NAME);
        info!("Removing '{}'.", path.display());
        if let Err(error) = fs::remove_file(&path) {
            match error.kind() {
                io::ErrorKind::NotFound => (),
                _ => return Err(Box::new(row::Error::FileRemove(path.clone(), error))),
            }
        }
    }

    Ok(())
}
