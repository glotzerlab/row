use clap::Args;
use console::style;
use indicatif::{HumanCount, HumanDuration};
use log::{debug, info, trace, warn};
use signal_hook::consts::{SIGINT, SIGTERM};
use signal_hook::flag;
use std::error::Error;
use std::io::prelude::*;
use std::io::{self, IsTerminal};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Instant;
use wildmatch::WildMatch;

use crate::cli::GlobalOptions;
use row::project::Project;
use row::scheduler::bash::Bash;
use row::scheduler::Scheduler;
use row::workflow::{Action, ResourceCost};
use row::MultiProgressContainer;

#[derive(Args, Debug)]
pub struct SubmitArgs {
    /// Select the actions to summarize with a wildcard pattern.
    #[arg(short, long, value_name = "pattern", default_value_t=String::from("*"), display_order=0)]
    action: String,

    /// Check the job submission status on the given cluster. Autodetected by default.
    #[arg(long, env = "ROW_CLUSTER", display_order = 0)]
    cluster: Option<String>,

    /// Select directories to summarize (defaults to all).
    directories: Vec<PathBuf>,

    /// Skip confirmation check.
    #[arg(long, display_order = 0, env = "ROW_YES", hide_env = true)]
    yes: bool,

    /// Print the scripts instead of submitting them.
    #[arg(long, display_order = 0)]
    dry_run: bool,

    /// Maximum number of jobs to submit.
    #[arg(short, display_order = 0)]
    n: Option<usize>,
}

/// Submit workflow actions to the scheduler.
///
pub fn submit<W: Write>(
    options: GlobalOptions,
    args: SubmitArgs,
    multi_progress: &mut MultiProgressContainer,
    output: &mut W,
) -> Result<(), Box<dyn Error>> {
    debug!("Submitting workflow actions to the scheduler.");
    let action_matcher = WildMatch::new(&args.action);

    let mut project = Project::open(options.io_threads, multi_progress)?;

    let query_directories = if args.directories.is_empty() {
        project.state().list_directories()
    } else {
        args.directories
    };

    let mut matching_action_count = 0;
    let mut action_groups: Vec<(&Action, Vec<Vec<PathBuf>>)> =
        Vec::with_capacity(project.workflow().action.len());

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
        let groups = project.separate_into_groups(action, status.eligible)?;

        action_groups.push((&action, groups));
    }

    if matching_action_count == 0 {
        warn!("No actions match '{}'.", args.action);
        project.close(multi_progress)?;
        return Ok(());
    }

    let mut total_cost = ResourceCost::new();
    let mut action_directories: Vec<(Action, Vec<PathBuf>)> = Vec::new();
    for (action, groups) in action_groups {
        let mut cost = ResourceCost::new();
        let mut job_count = 0;
        for group in groups {
            if let Some(n) = args.n {
                if action_directories.len() >= n {
                    break;
                }
            }

            cost = cost + action.resources.cost(group.len());
            action_directories.push((action.clone(), group.clone()));
            job_count += 1;
        }

        if job_count > 0 {
            info!(
                "Preparing {} {} that may cost up to {} for action '{}'.",
                job_count,
                if job_count == 1 { "job" } else { "jobs" },
                cost,
                action.name
            )
        }
        total_cost = total_cost + cost;

        if let Some(n) = args.n {
            if action_directories.len() >= n {
                break;
            }
        }
    }

    if action_directories.is_empty() {
        warn!("There are no eligible jobs to submit.");
        project.close(multi_progress)?;
        return Ok(());
    }

    // TODO: Validate submit_whole

    // TODO: Move scheduler into project, which will dynamically create a scheduler depending on the
    // detected cluster.
    let scheduler = Bash::new(&args.cluster.unwrap_or("none".into()));

    if args.dry_run {
        info!("Would submit the following scripts...");
        for (index, (action, directories)) in action_directories.iter().enumerate() {
            info!("script {}/{}:", index + 1, action_directories.len());
            let script = scheduler.make_script(action, directories)?;

            write!(output, "{}", script)?;
            output.flush()?;
        }
        project.close(multi_progress)?;
        return Ok(());
    }

    write!(output, "Submitting ")?;
    let jobs = if action_directories.len() == 1 {
        "job"
    } else {
        "jobs"
    };
    write!(
        output,
        "{} ",
        style(format!(
            "{} {}",
            HumanCount(action_directories.len() as u64),
            jobs
        ))
        .yellow()
        .bold()
    )?;

    writeln!(
        output,
        "that may cost up to {}.",
        style(total_cost).cyan().bold()
    )?;
    output.flush()?;

    if std::io::stdout().is_terminal() && !args.yes {
        let mut input = String::new();
        multi_progress.multi_progress.suspend(|| {
            print!("Proceed? [Y/n]: ");
            io::stdout().flush().expect("Can flush stdout");
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");
        });

        let selection = input.trim().to_lowercase();
        if selection != "y" && !selection.is_empty() {
            warn!("Cancelling submission.");
            return Ok(());
        }
    }

    // We are about to spawn child processes with user-defined input and output.
    // 1) Save the project cache now. Any user input error should not result
    //    in an out of date cache.
    // 2) Clear out the progress bars to allow the spawned processes stdout
    //    and/or stderr to go directly to the terminal.
    // 3) Stop using the buffered output and sync up all outputs by using
    //    stdin and stdout directly.
    project.close(multi_progress)?;

    multi_progress.progress_bars.clear();
    multi_progress.multi_progress.clear().unwrap();

    // Install the Ctrl-C signal handler to gracefully kill spawned processes
    // and save the pending scheduled job cache before exiting. Allow the user
    // to force an immediate shutdown with a 2nd Ctrl-C.
    // Make sure double CTRL+C and similar kills
    let should_terminate = Arc::new(AtomicBool::new(false));
    flag::register_conditional_shutdown(SIGINT, 10, Arc::clone(&should_terminate))?;
    flag::register(SIGINT, Arc::clone(&should_terminate))?;
    flag::register_conditional_shutdown(SIGTERM, 10, Arc::clone(&should_terminate))?;
    flag::register(SIGTERM, Arc::clone(&should_terminate))?;
    let instant = Instant::now();

    for (index, (action, directories)) in action_directories.iter().enumerate() {
        let mut message = format!(
            "[{}/{}] Submitting action '{}' on directory {}",
            HumanCount((index + 1) as u64),
            HumanCount(action_directories.len() as u64),
            style(action.name.clone()).blue(),
            style(directories[0].display().to_string()).bold()
        );
        if directories.len() > 1 {
            message += &style(format!(" and {} more", directories.len() - 1))
                .italic()
                .to_string();
        }
        message += &format!(" ({}).", style(HumanDuration(instant.elapsed())).dim());
        println!("{message}");

        // TODO: Change to the project directory before submitting.

        let result = scheduler.submit(
            &project.workflow().root,
            action,
            directories,
            Arc::clone(&should_terminate),
        );

        // TODO: Implement the job ID store. Save it after all jobs are
        // submitted, and when an error occurs. Need to capture successfully
        // submitted jobs in the store even when later jobs fail.
        if let Err(error) = result {
            return Err(error.into());
        }
    }

    Ok(())
}
