use clap::Args;
use console::Style;
use log::debug;
use std::collections::HashSet;
use std::error::Error;
use std::io::Write;
use std::path::PathBuf;

use crate::cli::{self, GlobalOptions};
use crate::ui::{Alignment, Item, Table};
use row::project::Project;
use row::MultiProgressContainer;

#[derive(Args, Debug)]
pub struct Arguments {
    /// Select the action to scan (defaults to all).
    action: String,

    /// Select directories to summarize (defaults to all). Use 'show directories -' to read from stdin.
    directories: Vec<PathBuf>,

    /// Hide the table header.
    #[arg(long, display_order = 0)]
    no_header: bool,

    /// Do not separate groups with newlines.
    #[arg(long, display_order = 0)]
    no_separate_groups: bool,

    /// Show an element of each directory's value (repeat to show multiple elements).
    #[arg(long, value_name = "JSON POINTER", display_order = 0)]
    value: Vec<String>,
}

/// Show directories that match an action.
///
/// Print a human-readable list of directories, their status, job ID, and value(s).
///
pub fn directories<W: Write>(
    options: &GlobalOptions,
    args: Arguments,
    multi_progress: &mut MultiProgressContainer,
    output: &mut W,
) -> Result<(), Box<dyn Error>> {
    debug!("Showing directories.");

    let mut project = Project::open(options.io_threads, &options.cluster, multi_progress)?;

    let query_directories =
        cli::parse_directories(args.directories, || Ok(project.state().list_directories()))?;

    let action = project
        .workflow()
        .action_by_name(&args.action)
        .ok_or_else(|| row::Error::ActionNotFound(args.action))?;

    let matching_directories =
        project.find_matching_directories(action, query_directories.clone())?;

    let status = project.separate_by_status(action, matching_directories.clone())?;
    let completed = HashSet::<PathBuf>::from_iter(status.completed);
    let submitted = HashSet::<PathBuf>::from_iter(status.submitted);
    let eligible = HashSet::<PathBuf>::from_iter(status.eligible);
    let waiting = HashSet::<PathBuf>::from_iter(status.waiting);

    // TODO: filter shown directories by status, also add --n_groups option
    let groups = project.separate_into_groups(action, matching_directories)?;

    let mut table = Table::new().with_hide_header(args.no_header);
    table.header = vec![
        Item::new("Directory".to_string(), Style::new().underlined()),
        Item::new("Status".to_string(), Style::new().underlined()),
        Item::new("Job ID".to_string(), Style::new().underlined()),
    ];
    for pointer in &args.value {
        table
            .header
            .push(Item::new(pointer.clone(), Style::new().underlined()));
    }

    for (group_idx, group) in groups.iter().enumerate() {
        for directory in group {
            // Format the directory status.
            let status = if completed.contains(directory) {
                Item::new("completed".to_string(), Style::new().green().italic())
            } else if submitted.contains(directory) {
                Item::new("submitted".to_string(), Style::new().yellow().italic())
            } else if eligible.contains(directory) {
                Item::new("eligible".to_string(), Style::new().blue().italic())
            } else if waiting.contains(directory) {
                Item::new("waiting".to_string(), Style::new().cyan().dim().italic())
            } else {
                panic!("Directory not found in status.")
            };

            let mut row = Vec::new();

            // The directory name
            row.push(Item::new(
                directory.display().to_string(),
                Style::new().bold(),
            ));

            // Status
            row.push(status);

            // Job ID
            let submitted = project.state().submitted();

            // Values
            if let Some((cluster, job_id)) =
                submitted.get(&action.name).and_then(|d| d.get(directory))
            {
                row.push(Item::new(format!("{cluster}/{job_id}"), Style::new()));
            } else {
                row.push(Item::new(String::new(), Style::new()));
            }
            for pointer in &args.value {
                let value = project.state().values()[directory]
                    .pointer(pointer)
                    .ok_or_else(|| {
                        row::Error::JSONPointerNotFound(directory.clone(), pointer.clone())
                    })?;
                row.push(
                    Item::new(value.to_string(), Style::new()).with_alignment(Alignment::Right),
                );
            }

            table.items.push(row);
        }

        if !args.no_separate_groups && group_idx != groups.len() - 1 {
            let mut row = vec![
                Item::new(String::new(), Style::new()),
                Item::new(String::new(), Style::new()),
            ];
            for _ in &args.value {
                row.push(Item::new(String::new(), Style::new()));
            }
            table.items.push(row);
        }
    }

    table.write(output)?;
    output.flush()?;

    project.close(multi_progress)?;

    Ok(())
}
