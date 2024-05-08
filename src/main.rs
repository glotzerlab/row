#![warn(clippy::pedantic)]

use clap::Parser;
use indicatif::{MultiProgress, ProgressDrawTarget};
use indicatif_log_bridge::LogWrapper;
use log::{error, info};
use std::error::Error;
use std::io::{self, Write};
use std::process::ExitCode;
use std::time::Instant;

mod cli;
mod ui;

use cli::{ColorMode, Commands, Options, ShowCommands};
use row::format::HumanDuration;
use row::MultiProgressContainer;
use ui::MultiProgressWriter;

fn main_detail() -> Result<(), Box<dyn Error>> {
    let instant = Instant::now();
    let options = Options::parse();

    let log_style;
    match options.global.color {
        ColorMode::Never => {
            log_style = "never";
            console::set_colors_enabled(false);
        }
        ColorMode::Always => {
            log_style = "always";
            console::set_colors_enabled(true);
        }
        ColorMode::Auto => {
            log_style = "auto";
        }
    }

    let log_level = match options.verbose.log_level_filter() {
        clap_verbosity_flag::LevelFilter::Off => "off",
        clap_verbosity_flag::LevelFilter::Error => "error",
        clap_verbosity_flag::LevelFilter::Warn => "warn",

        clap_verbosity_flag::LevelFilter::Info => "info",
        clap_verbosity_flag::LevelFilter::Debug => "debug",
        clap_verbosity_flag::LevelFilter::Trace => "trace",
    };

    let multi_progress = if options.global.no_progress {
        MultiProgress::with_draw_target(ProgressDrawTarget::hidden())
    } else {
        MultiProgress::new()
    };

    let mut output = MultiProgressWriter::new(io::stdout(), multi_progress.clone());

    let env = env_logger::Env::default()
        .filter_or("ROW_LOG", log_level)
        .write_style_or("ROW_LOG_STYLE", log_style);

    let logger = env_logger::Builder::from_env(env)
        .format_timestamp(None)
        .build();

    LogWrapper::new(multi_progress.clone(), logger).try_init()?;

    let mut multi_progress_container = MultiProgressContainer::new(multi_progress.clone());

    match options.command {
        Some(Commands::Show(show)) => match show {
            ShowCommands::Status(args) => cli::status::status(
                &options.global,
                args,
                &mut multi_progress_container,
                &mut output,
            )?,
            ShowCommands::Directories(args) => cli::directories::directories(
                &options.global,
                args,
                &mut multi_progress_container,
                &mut output,
            )?,
            ShowCommands::Cluster(args) => {
                cli::cluster::cluster(&options.global, &args, &mut output)?;
            }
            ShowCommands::Launchers(args) => {
                cli::launchers::launchers(&options.global, &args, &mut output)?;
            }
        },
        Some(Commands::Scan(args)) => {
            cli::scan::scan(&options.global, args, &mut multi_progress_container)?;
        }
        Some(Commands::Submit(args)) => cli::submit::submit(
            &options.global,
            args,
            &mut multi_progress_container,
            &mut output,
        )?,
        None => (),
    }

    // Drop output here - otherwise it is dropped after multi_progress and the progress bars
    // are always cleared on exit.
    output.flush()?;
    drop(output);

    info!("Completed in {}.", HumanDuration(instant.elapsed()));

    if options.global.clear_progress {
        multi_progress.clear().unwrap();
    }

    Ok(())
}

fn main() -> ExitCode {
    if let Err(error) = main_detail() {
        error!("{error}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
