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
