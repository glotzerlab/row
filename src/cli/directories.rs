// Copyright (c) 2024 The Regents of the University of Michigan.
// Part of row, released under the BSD 3-Clause License.

use clap::Args;
use console::Style;
use log::{debug, warn};
use std::collections::HashSet;
use std::error::Error;
use std::io::Write;
use std::path::PathBuf;

use crate::cli::{self, GlobalOptions};
use crate::ui::{Alignment, Item, Row, Table};
use row::project::Project;
use row::MultiProgressContainer;

#[derive(Args, Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct Arguments {
    /// Select directories to summarize (defaults to all). Use 'show directories -' to read from stdin.
    directories: Vec<PathBuf>,

    /// Select directories that are included by the provided action.
    #[arg(long, short, display_order = 0)]
    action: Option<String>,

    /// Hide the table header.
    #[arg(long, display_order = 0, requires = "action")]
    no_header: bool,

    /// Do not separate groups with newlines.
    #[arg(long, display_order = 0, requires = "action")]
    no_separate_groups: bool,

    /// Show an element of each directory's value (repeat to show multiple elements).
    #[arg(
        long,
        value_name = "JSON POINTER",
        display_order = 0,
        requires = "action"
    )]
    value: Vec<String>,

    /// Limit the number of groups displayed.
    #[arg(short, long, display_order = 0, requires = "action")]
    n_groups: Option<usize>,

    /// Show completed directories.
    #[arg(long, display_order = 0, requires = "action")]
    completed: bool,

    /// Show submitted directories.
    #[arg(long, display_order = 0, requires = "action")]
    submitted: bool,

    /// Show eligible directories.
    #[arg(long, display_order = 0, requires = "action")]
    eligible: bool,

    /// Show waiting directories.
    #[arg(long, display_order = 0, requires = "action")]
    waiting: bool,

    /// Show only directory names.
    #[arg(long, default_value_t = false, display_order = 0, requires = "action")]
    short: bool,
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
    match &args.action {
        Some(action) => print_matching(&action.clone(), options, args, multi_progress, output),
        None => print_all(options, args, multi_progress, output),
    }
}

#[allow(clippy::too_many_lines)]
pub fn print_matching<W: Write>(
    action_name: &str,
    options: &GlobalOptions,
    args: Arguments,
    multi_progress: &mut MultiProgressContainer,
    output: &mut W,
) -> Result<(), Box<dyn Error>> {
    // Show directories with selected statuses.
    let mut show_completed = args.completed;
    let mut show_submitted = args.submitted;
    let mut show_eligible = args.eligible;
    let mut show_waiting = args.waiting;
    if !show_completed && !show_submitted && !show_eligible && !show_waiting {
        show_completed = true;
        show_submitted = true;
        show_eligible = true;
        show_waiting = true;
    }

    let mut project = Project::open(options.io_threads, &options.cluster, multi_progress)?;

    let query_directories =
        cli::parse_directories(args.directories, || Ok(project.state().list_directories()))?;

    project
        .workflow()
        .action_by_name(action_name)
        .ok_or_else(|| row::Error::ActionNotFound(action_name.to_string()))?;

    let mut table = Table::new().with_hide_header(if args.short { true } else { args.no_header });
    table.header = vec![
        Item::new("Directory".to_string(), Style::new().underlined()),
        Item::new("Status".to_string(), Style::new().underlined()),
    ];
    if show_submitted || show_completed {
        table
            .header
            .push(Item::new("Job ID".to_string(), Style::new().underlined()));
    }
    for pointer in &args.value {
        table
            .header
            .push(Item::new(pointer.clone(), Style::new().underlined()));
    }

    for action in &project.workflow().action {
        if action.name() != action_name {
            continue;
        }

        let matching_directories =
            project.find_matching_directories(action, query_directories.clone())?;

        let status = project.separate_by_status(action, matching_directories.clone())?;
        let completed = HashSet::<PathBuf>::from_iter(status.completed.clone());
        let submitted = HashSet::<PathBuf>::from_iter(status.submitted.clone());
        let eligible = HashSet::<PathBuf>::from_iter(status.eligible.clone());
        let waiting = HashSet::<PathBuf>::from_iter(status.waiting.clone());

        let mut selected_directories = Vec::with_capacity(matching_directories.len());
        if show_completed {
            selected_directories.extend(status.completed);
        }
        if show_submitted {
            selected_directories.extend(status.submitted);
        }
        if show_eligible {
            selected_directories.extend(status.eligible);
        }
        if show_waiting {
            selected_directories.extend(status.waiting);
        }

        let groups = project.separate_into_groups(action, selected_directories)?;

        for (group_idx, group) in groups.iter().enumerate() {
            if let Some(n) = args.n_groups {
                if group_idx >= n {
                    break;
                }
            }

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

                // Only show directory names when user requests short output.
                if args.short {
                    table.rows.push(Row::Items(row));
                    continue;
                }

                // Status
                row.push(status);

                // Job ID
                if show_submitted || show_completed {
                    let submitted = project.state().submitted();

                    // Values
                    if let Some((cluster, job_id)) =
                        submitted.get(action.name()).and_then(|d| d.get(directory))
                    {
                        row.push(Item::new(format!("{cluster}/{job_id}"), Style::new()));
                    } else {
                        row.push(Item::new(String::new(), Style::new()));
                    }
                }

                for pointer in &args.value {
                    if !pointer.is_empty() && !pointer.starts_with('/') {
                        warn!("The JSON pointer '{pointer}' does not appear valid. Did you mean '/{pointer}'?");
                    }

                    let value = project.state().values()[directory]
                        .pointer(pointer)
                        .ok_or_else(|| {
                            row::Error::JSONPointerNotFound(directory.clone(), pointer.clone())
                        })?;
                    row.push(
                        Item::new(value.to_string(), Style::new()).with_alignment(Alignment::Right),
                    );
                }

                table.rows.push(Row::Items(row));
            }

            if !args.no_separate_groups && group_idx != groups.len() - 1 && !args.short {
                table.rows.push(Row::Separator);
            }
        }

        if !args.short {
            table.rows.push(Row::Separator);
        }
    }

    table.write(output)?;
    output.flush()?;

    project.close(multi_progress)?;

    Ok(())
}

pub fn print_all<W: Write>(
    options: &GlobalOptions,
    args: Arguments,
    multi_progress: &mut MultiProgressContainer,
    output: &mut W,
) -> Result<(), Box<dyn Error>> {
    let project = Project::open(options.io_threads, &options.cluster, multi_progress)?;

    let all_directories = project.state().list_directories();
    let mut query_directories =
        cli::parse_directories(args.directories, || Ok(all_directories.clone()))?;
    query_directories.sort_unstable();
    let all_directories = HashSet::<PathBuf>::from_iter(all_directories);

    for directory in &query_directories {
        if !all_directories.contains(directory) {
            warn!(
                "Directory '{}' not found in workspace.",
                directory.display()
            );
            continue;
        }
        writeln!(output, "{}", directory.display())?;
    }

    Ok(())
}
