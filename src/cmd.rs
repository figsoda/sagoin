use anyhow::{anyhow, Result};

use std::{ffi::OsString, process::Command};

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

pub(crate) fn run_hook(cmd: &Option<OsString>, name: &'static str) -> Result<()> {
    match cmd {
        Some(cmd) if !cmd.is_empty() => {
            eprintln!("Running {name} hook");
            let status = shell().arg(cmd).status()?;
            if !status.success() {
                Err(anyhow!("{name} hook failed with exit code {status}"))
            } else {
                Ok(())
            }
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::{run_hook, shell};

    #[test]
    fn shell_echo() {
        let output = shell().arg("echo foo").output().unwrap();
        assert!(output.status.success());
        assert!(output.stdout.starts_with(b"foo"));
    }

    #[test]
    fn run_hook_none() {
        assert!(run_hook(&None, "").is_ok());
        assert!(run_hook(&Some("".into()), "").is_ok());
    }

    #[test]
    fn run_hook_echo() {
        assert!(run_hook(&Some("echo foo".into()), "").is_ok());
    }
}
