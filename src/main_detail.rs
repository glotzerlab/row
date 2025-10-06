// Copyright (c) 2024-2025 The Regents of the University of Michigan.
// Part of row, released under the BSD 3-Clause License.

use clap::{CommandFactory, Parser};
use clap_complete::env::CompleteEnv;
use clap_verbosity_flag::log::LevelFilter;
use indicatif::{MultiProgress, ProgressDrawTarget};
use indicatif_log_bridge::LogWrapper;
use log::info;
use std::error::Error;
use std::io::{self, Write};
use std::time::Instant;

use crate::cli;
use crate::ui;

use cli::{ColorMode, Commands, Options, ShowCommands};
use crate::MultiProgressContainer;
use crate::format::HumanDuration;
use ui::MultiProgressWriter;

pub(crate) fn main_detail() -> Result<(), Box<dyn Error>> {
    let instant = Instant::now();

    // Autocomplete
    CompleteEnv::with_factory(Options::command).complete();

    // Normal execution
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
        LevelFilter::Off => "off",
        LevelFilter::Error => "error",
        LevelFilter::Warn => "warn",

        LevelFilter::Info => "info",
        LevelFilter::Debug => "debug",
        LevelFilter::Trace => "trace",
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
        Some(Commands::Init(args)) => {
            cli::init::init(&options.global, &args, &mut output)?;
        }
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
            ShowCommands::Jobs(args) => {
                cli::jobs::show(
                    &options.global,
                    args,
                    &mut multi_progress_container,
                    &mut output,
                )?;
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
        Some(Commands::Clean(args)) => {
            cli::clean::clean(&options.global, &args, &mut multi_progress_container)?;
        }
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
