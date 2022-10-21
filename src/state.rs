use std::{
    fmt::Display,
    io::{self, StderrLock, Write},
};

use anyhow::{Context, Result};

pub struct State<W: Write> {
    pub(crate) color: bool,
    pub(crate) out: W,
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
    pub fn stderr() -> Result<Self> {
        Ok(Self {
            color: concolor::get(concolor::Stream::Stderr).ansi_color(),
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
        self.out.flush().context("Failed to flush stderr")?;
        Ok(())
    }
}

#[cfg(test)]
impl State<io::Sink> {
    pub(crate) fn sink() -> State<io::Sink> {
        Self {
            color: false,
            out: io::sink(),
        }
    }
}
