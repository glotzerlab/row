// Copyright (c) 2024-2025 The Regents of the University of Michigan.
// Part of row, released under the BSD 3-Clause License.

use clap::Args;
use clap_complete::ArgValueCandidates;
use console::Style;
use log::{debug, trace};
use std::collections::{BTreeMap, HashSet};
use std::error::Error;
use std::io::Write;
use std::path::PathBuf;
use wildmatch::WildMatch;

use crate::cli::{self, autocomplete, GlobalOptions};
use crate::ui::{Alignment, Item, Row, Table};
use row::project::Project;
use row::MultiProgressContainer;

#[derive(Args, Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct Arguments {
    /// Show jobs running on these directories (defaults to all). Use 'show jobs -' to read from stdin.
    #[arg(add=ArgValueCandidates::new(autocomplete::get_directory_candidates))]
    directories: Vec<PathBuf>,

    /// Show jobs running actions that match a wildcard pattern.
    #[arg(short, long, value_name = "pattern", default_value_t=String::from("*"), display_order=0,
        add=ArgValueCandidates::new(autocomplete::get_action_candidates))]
    action: String,

    /// Hide the table header.
    #[arg(long, display_order = 0)]
    no_header: bool,

    /// Show only job IDs.
    #[arg(long, default_value_t = false, display_order = 0)]
    short: bool,
}

struct JobDetails {
    action: String,
    n: u64,
}

/** Find jobs that match the given directories and the action wildcard on the selected cluster.
*/
fn find(
    directories: Vec<PathBuf>,
    action: &str,
    project: &Project,
) -> Result<BTreeMap<u32, JobDetails>, Box<dyn Error>> {
    debug!("Finding matching jobs.");
    let mut result: BTreeMap<u32, JobDetails> = BTreeMap::new();

    let action_matcher = WildMatch::new(action);

    let query_directories: HashSet<PathBuf> =
        HashSet::from_iter(cli::parse_directories(directories, || {
            Ok(project.state().list_directories())
        })?);

    for (action_name, jobs_by_directory) in project.state().submitted() {
        if !action_matcher.matches(action_name) {
            trace!(
                "Skipping action '{}'. It does not match the pattern '{}'.",
                action_name,
                action
            );
            continue;
        }

        for (directory_name, (cluster_name, job_id)) in jobs_by_directory {
            if cluster_name != project.cluster_name() {
                trace!(
                    "Skipping cluster '{cluster_name}'. It does not match selected cluster '{}'.",
                    project.cluster_name()
                );
                continue;
            }

            if query_directories.contains(directory_name) {
                result
                    .entry(*job_id)
                    .and_modify(|e| e.n += 1)
                    .or_insert(JobDetails {
                        action: action_name.clone(),
                        n: 1,
                    });
            }
        }
    }

    Ok(result)
}

/** Show jobs running on given directories where the action also matches a wildcard.

Print a human-readable list of job IDs, the action they are running, and the number of
directories that the job acts on.
*/
pub fn show<W: Write>(
    options: &GlobalOptions,
    args: Arguments,
    multi_progress: &mut MultiProgressContainer,
    output: &mut W,
) -> Result<(), Box<dyn Error>> {
    debug!("Showing jobs.");

    let mut project = Project::open(options.io_threads, &options.cluster, multi_progress)?;

    let jobs = find(args.directories, &args.action, &project)?;

    let mut table = Table::new().with_hide_header(if args.short { true } else { args.no_header });
    table.header = vec![
        Item::new("ID".to_string(), Style::new().underlined()),
        Item::new("Action".to_string(), Style::new().underlined()),
        Item::new("Directories".to_string(), Style::new().underlined()),
    ];

    for (job_id, job_details) in jobs {
        let mut row = Vec::new();

        row.push(
            Item::new(job_id.to_string(), Style::new().bold()).with_alignment(Alignment::Right),
        );

        // Only show job IDs when user requests short output.
        if args.short {
            table.rows.push(Row::Items(row));
            continue;
        }

        row.push(Item::new(job_details.action, Style::new()));
        row.push(
            Item::new(job_details.n.to_string(), Style::new()).with_alignment(Alignment::Right),
        );

        table.rows.push(Row::Items(row));
    }

    table.write(output)?;
    output.flush()?;

    project.close(multi_progress)?;

    Ok(())
}
