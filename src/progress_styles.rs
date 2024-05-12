// Copyright (c) 2024 The Regents of the University of Michigan.
// Part of row, released under the BSD 3-Clause License.

use indicatif::{ProgressState, ProgressStyle};
use std::fmt::Write;

use crate::format::HumanDuration;

pub(crate) const STEADY_TICK: u64 = 110;

/// Format progress duration in milliseconds
fn elapsed(state: &ProgressState, w: &mut dyn Write) {
    let _ = write!(w, "{:#}", HumanDuration(state.elapsed()));
}

/// Create a named spinner.
///
/// # Panics
/// When the progress style is invalid.
///
pub fn uncounted_spinner() -> ProgressStyle {
    ProgressStyle::with_template("{spinner:.green.bold} {msg:.bold}... ({elapsed:.dim})")
        .expect("Valid template")
        .with_key("elapsed", elapsed)
        .tick_strings(&["◐", "◓", "◑", "◒", "⊙"])
}

/// Create a spinner that displays the current counted position.
///
/// # Panics
/// When the progress style is invalid.
///
pub fn counted_spinner() -> ProgressStyle {
    ProgressStyle::with_template("{spinner:.green.bold} {msg:.bold}: {human_pos} ({elapsed:.dim})")
        .expect("Valid template")
        .with_key("elapsed", elapsed)
        .tick_strings(&["◐", "◓", "◑", "◒", "⊙"])
}

/// Create a progress bar that displays the current counted position.
///
/// # Panics
/// When the progress style is invalid.
///
pub fn counted_bar() -> ProgressStyle {
    ProgressStyle::with_template(
        "|{bar:32.green}| {msg:.bold}: {human_pos}/{human_len} ({elapsed:.dim})",
    )
    .expect("Valid template")
    .with_key("elapsed", elapsed)
    .progress_chars("█▉▊▋▌▍▎▏  ")
}
