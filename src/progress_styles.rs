use indicatif::ProgressStyle;

pub(crate) const STEADY_TICK: u64 = 110;

pub fn uncounted_spinner() -> ProgressStyle {
    ProgressStyle::with_template("{spinner:.green.bold} {msg:.bold}... ({elapsed:.dim})")
        .expect("Valid template")
        .tick_strings(&["◐", "◓", "◑", "◒", "⊙"])
}

pub fn counted_spinner() -> ProgressStyle {
    ProgressStyle::with_template("{spinner:.green.bold} {msg:.bold}: {human_pos} ({elapsed:.dim})")
        .expect("Valid template")
        .tick_strings(&["◐", "◓", "◑", "◒", "⊙"])
}

pub fn counted_bar() -> ProgressStyle {
    ProgressStyle::with_template(
        "|{bar:32.green}| {msg:.bold}: {human_pos}/{human_len} ({elapsed:.dim})",
    )
    .expect("Valid template")
    .progress_chars("█▉▊▋▌▍▎▏  ")
}
