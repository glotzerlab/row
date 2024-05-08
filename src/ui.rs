use console::Style;
use indicatif::MultiProgress;
use memchr::memmem;
use std::cmp;
use std::io::{self, Write};

/// The default writer buffer size.
const DEFAULT_BUFFER_SIZE: usize = 1024;

/// Buffered writer that interoperates with a `MultiProgress`.
///
/// Use this writer to buffer writes to stdout/stderr. When flushed, the
/// writer will suspend the `MultiProgress` and write the output.
///
pub struct MultiProgressWriter<T: Write> {
    inner: T,
    multi_progress: MultiProgress,
    buffer: Vec<u8>,
    buffer_size: usize,
}

impl<T: Write> MultiProgressWriter<T> {
    /// Create a new writer.
    ///
    /// # Arguments
    /// * `inner`: Writer to forward output to.
    /// * `multi_progress`: The `MultiProgress` to suspend when writing.
    ///
    pub fn new(inner: T, multi_progress: MultiProgress) -> Self {
        Self {
            inner,
            multi_progress,
            buffer: Vec::with_capacity(DEFAULT_BUFFER_SIZE),
            buffer_size: DEFAULT_BUFFER_SIZE,
        }
    }
}

impl<T: Write> Write for MultiProgressWriter<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.buffer.len() >= self.buffer_size {
            self.flush()?;
        }

        self.buffer.extend_from_slice(buf);

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        if let Some(last_newline) = memmem::rfind(&self.buffer, b"\n") {
            self.multi_progress.suspend(|| -> io::Result<()> {
                self.inner.write_all(&self.buffer[0..=last_newline])
            })?;
            self.buffer.drain(0..=last_newline);
            self.inner.flush()?;
        }
        Ok(())
    }
}

impl<T: Write> Drop for MultiProgressWriter<T> {
    fn drop(&mut self) {
        if !self.buffer.is_empty() {
            self.flush().unwrap();
        }
    }
}

pub(crate) enum Alignment {
    Left,
    Right,
}

/// One item in a table.
pub(crate) struct Item {
    text: String,
    style: Style,
    alignment: Alignment,
}

/// The table
pub(crate) struct Table {
    // The header row.
    pub header: Vec<Item>,

    // The items.
    pub items: Vec<Vec<Item>>,

    // Hide the header when true.
    hide_header: bool,
}

impl Item {
    pub(crate) fn new(text: String, style: Style) -> Self {
        Item {
            text,
            style,
            alignment: Alignment::Left,
        }
    }

    pub(crate) fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }
}

impl Table {
    pub(crate) fn new() -> Self {
        Table {
            header: Vec::new(),
            items: Vec::new(),
            hide_header: false,
        }
    }

    pub(crate) fn with_hide_header(mut self, hide_header: bool) -> Self {
        self.hide_header = hide_header;
        self
    }

    fn write_row<W: Write>(writer: &mut W, row: &[Item], column_width: &[usize]) -> io::Result<()> {
        for (i, item) in row.iter().enumerate() {
            let text = match item.alignment {
                Alignment::Left => format!("{:<width$}", &item.text, width = column_width[i]),
                Alignment::Right => format!("{:>width$}", &item.text, width = column_width[i]),
            };

            write!(writer, "{}", &item.style.apply_to(text))?;
            if i != row.len() - 1 {
                write!(writer, " ")?;
            }
        }

        writeln!(writer)?;

        Ok(())
    }

    pub(crate) fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        let mut column_width: Vec<usize> = self
            .header
            .iter()
            .map(|h| console::measure_text_width(&h.text))
            .collect();
        for row in &self.items {
            for (i, item) in row.iter().enumerate() {
                column_width[i] =
                    cmp::max(console::measure_text_width(&item.text), column_width[i]);
            }
        }

        if !self.hide_header {
            Self::write_row(writer, &self.header, &column_width)?;
        }

        for row in &self.items {
            Self::write_row(writer, row, &column_width)?;
        }

        Ok(())
    }
}
