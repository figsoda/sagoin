use std::{
    fmt::Display,
    io::{self, StderrLock, Write},
};

use color_eyre::config::{HookBuilder, Theme};
use eyre::{Result, WrapErr};

pub struct State<W: Write> {
    pub color: bool,
    pub out: W,
}

#[macro_export]
macro_rules! warn {
    ($self:expr, $($tt:tt)*) => {{
        if $self.color {
            write!($self.out, "\x1b[1;33mWarning\x1b[0m: ").unwrap();
        } else {
            write!($self.out, "Warning: ").unwrap();
        }

        writeln!($self.out, $($tt)*).unwrap();
    }};
}

impl State<StderrLock<'static>> {
    pub(crate) fn stderr() -> Result<Self> {
        let color = concolor::get(concolor::Stream::Stderr).ansi_color();

        if color {
            color_eyre::install()?;
        } else {
            HookBuilder::new().theme(Theme::new()).install()?;
        }

        Ok(Self {
            color,
            out: io::stderr().lock(),
        })
    }
}

impl<W: Write> State<W> {
    pub(crate) fn prompt(&mut self, p: impl Display) -> Result<()> {
        if self.color {
            write!(self.out, "\x1b[1;34m{p}\x1b[0m: ")?;
        } else {
            write!(self.out, "{p}: ")?;
        }

        self.out.flush().wrap_err("failed to flush stderr")?;

        Ok(())
    }
}

#[cfg(test)]
impl State<io::Sink> {
    pub(crate) fn sink() -> Self {
        Self {
            color: false,
            out: io::sink(),
        }
    }
}

#[cfg(test)]
impl State<Vec<u8>> {
    pub(crate) fn buffer() -> Self {
        Self {
            color: false,
            out: Vec::new(),
        }
    }
}
