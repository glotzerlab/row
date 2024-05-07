use clap::Args;
use console::Style;
use indicatif::HumanCount;
use log::{debug, trace, warn};
use std::error::Error;
use std::io::Write;
use std::path::PathBuf;
use wildmatch::WildMatch;

use crate::cli::{self, GlobalOptions};
use crate::ui::{Alignment, Item, Table};
use row::project::{Project, Status};
use row::workflow::ResourceCost;
use row::MultiProgressContainer;

#[derive(Args, Debug)]
pub struct StatusArgs {
    /// Select the actions to summarize with a wildcard pattern.
    #[arg(short, long, value_name = "pattern", default_value_t=String::from("*"), display_order=0)]
    action: String,

    /// Hide the table header.
    #[arg(long, display_order = 0)]
    no_header: bool,

    /// Select directories to summarize (defaults to all). Use 'status -' to read from stdin.
    directories: Vec<PathBuf>,
}

/// Format a status string for non-terminal outputs.
fn make_row(action_name: &str, status: &Status, cost: ResourceCost) -> Vec<Item> {
    let mut result = Vec::with_capacity(6);
    result.push(Item::new(action_name.to_string(), Style::new().bold()));
    result.push(
        Item::new(
            HumanCount(status.completed.len() as u64).to_string(),
            Style::new().green().bold(),
        )
        .with_alignment(Alignment::Right),
    );
    result.push(
        Item::new(
            HumanCount(status.submitted.len() as u64).to_string(),
            Style::new().yellow().bold(),
        )
        .with_alignment(Alignment::Right),
    );
    result.push(
        Item::new(
            HumanCount(status.eligible.len() as u64).to_string(),
            Style::new().blue(),
        )
        .with_alignment(Alignment::Right),
    );
    result.push(
        Item::new(
            HumanCount(status.waiting.len() as u64).to_string(),
            Style::new().cyan().dim(),
        )
        .with_alignment(Alignment::Right),
    );

    if !cost.is_zero() {
        result.push(
            Item::new(format!("{}", cost), Style::new().italic().dim())
                .with_alignment(Alignment::Right),
        );
    }

    result
}

/// Show the current state of the workflow.
///
/// Print a human-readable summary of the workflow.
///
pub fn status<W: Write>(
    options: GlobalOptions,
    args: StatusArgs,
    multi_progress: &mut MultiProgressContainer,
    output: &mut W,
) -> Result<(), Box<dyn Error>> {
    debug!("Showing the workflow's status.");
    let action_matcher = WildMatch::new(&args.action);

    let mut project = Project::open(options.io_threads, options.cluster, multi_progress)?;

    let query_directories =
        cli::parse_directories(args.directories, || Ok(project.state().list_directories()))?;

    let mut table = Table::new().with_hide_header(args.no_header);
    let underlined = Style::new().underlined();
    table.header = vec![
        Item::new("Action".to_string(), underlined.clone()),
        Item::new("Completed".to_string(), underlined.clone()).with_alignment(Alignment::Right),
        Item::new("Submitted".to_string(), underlined.clone()).with_alignment(Alignment::Right),
        Item::new("Eligible".to_string(), underlined.clone()).with_alignment(Alignment::Right),
        Item::new("Waiting".to_string(), underlined.clone()).with_alignment(Alignment::Right),
        Item::new("Remaining cost".to_string(), underlined.clone())
            .with_alignment(Alignment::Right),
    ];

    let mut matching_action_count = 0;
    for action in &project.workflow().action {
        if !action_matcher.matches(&action.name) {
            trace!(
                "Skipping action '{}'. It does not match the pattern '{}'.",
                action.name,
                args.action
            );
            continue;
        }

        matching_action_count += 1;

        let matching_directories =
            project.find_matching_directories(action, query_directories.clone())?;

        let status = project.separate_by_status(action, matching_directories)?;

        let mut combined_directories = Vec::with_capacity(
            status.submitted.len() + status.eligible.len() + status.waiting.len(),
        );
        combined_directories.extend(status.submitted.clone());
        combined_directories.extend(status.eligible.clone());
        combined_directories.extend(status.waiting.clone());

        let groups = project.separate_into_groups(action, combined_directories.clone())?;
        let mut cost = ResourceCost::new();
        for group in groups {
            cost = cost + action.resources.cost(group.len());
        }

        table.items.push(make_row(&action.name, &status, cost));
    }

    if matching_action_count == 0 {
        warn!("No actions match '{}'.", args.action);
    } else {
        table.write(output)?;
        output.flush()?;
    }

    project.close(multi_progress)?;

    Ok(())
}
