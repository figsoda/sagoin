use anyhow::{Context, Result};
use rpassword::read_password;

use std::{
    fs::read_to_string,
    io::{self, Write},
    process::{Output, Stdio},
};

use crate::{cli::InputType, cmd, Opts};

pub(crate) fn resolve_username(opts: &Opts) -> Result<String> {
    Ok(
        if let Some(user) = resolve_cred(&opts.username, opts.username_type) {
            user
        } else {
            eprint!("Username: ");
            io::stderr()
                .flush()
                .context("Failed to prompt for username")?;

            let mut user = String::new();
            io::stdin()
                .read_line(&mut user)
                .context("Failed to prompt for username")?;

            user
        },
    )
}

pub(crate) fn resolve_password(opts: &Opts) -> Result<String> {
    if let Some(pass) = resolve_cred(&opts.password, opts.password_type) {
        Ok(pass)
    } else {
        eprint!("Password: ");
        io::stderr()
            .flush()
            .context("Failed to prompt for password")?;
        read_password().context("Failed to prompt for password")
    }
}

fn resolve_cred(cred: &Option<String>, t: InputType) -> Option<String> {
    cred.as_ref().and_then(|input| {
        if input.is_empty() {
            None
        } else {
            match t {
                InputType::Text => Some(input.into()),
                InputType::File => read_to_string(input)
                    .map_err(|e| eprintln!("Warning: Failed to read {input}:\n{e}"))
                    .ok(),
                InputType::Command => cmd::shell()
                    .arg(input)
                    .stderr(Stdio::inherit())
                    .stdin(Stdio::inherit())
                    .output()
                    .map_err(|e| eprintln!("Warning: Failed to execute command: {e}"))
                    .ok()
                    .and_then(|Output { status, stdout, .. }| {
                        if status.success() {
                            String::from_utf8(stdout)
                                .map_err(|e| eprintln!("Warning: {e}"))
                                .ok()
                        } else {
                            eprintln!("Warning: command failed with exit code {status}");
                            None
                        }
                    }),
            }
        }
    })
}
