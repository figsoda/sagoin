use std::{ffi::OsString, io::Write, process::Command};

use eyre::{eyre, Result};

use crate::state::State;

#[cfg(unix)]
pub(crate) fn shell() -> Command {
    let mut cmd = Command::new("sh");
    cmd.arg("-c");
    cmd
}

#[cfg(windows)]
pub(crate) fn shell() -> Command {
    let mut cmd = Command::new("cmd.exe");
    cmd.arg("/c");
    cmd
}

impl<W: Write> State<W> {
    pub(crate) fn run_hook(&mut self, cmd: &Option<OsString>, name: &'static str) -> Result<()> {
        match cmd {
            Some(cmd) if !cmd.is_empty() => {
                writeln!(self.out, "Running {name} hook")?;
                let status = shell().arg(cmd).status()?;
                if !status.success() {
                    Err(eyre!("{name} hook failed with exit code {status}"))
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::shell;
    use crate::state::State;

    #[test]
    fn shell_echo() {
        let output = shell().arg("echo foo").output().unwrap();
        assert!(output.status.success());
        assert!(output.stdout.starts_with(b"foo"));
    }

    #[test]
    fn run_hook_none() {
        let mut state = State::buffer();

        assert!(state.run_hook(&None, "foo").is_ok());
        assert!(state.out.is_empty());

        assert!(state.run_hook(&Some("".into()), "foo").is_ok());
        assert!(state.out.is_empty());
    }

    #[test]
    fn run_hook_echo() {
        let mut state = State::buffer();
        assert!(state.run_hook(&Some("echo foo".into()), "bar").is_ok());
        assert!(std::str::from_utf8(&state.out).unwrap().contains("bar"));
    }
}
