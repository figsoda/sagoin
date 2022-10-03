use anyhow::{Context, Result};
use rpassword::prompt_password;

use std::{
    fs::read_to_string,
    io::{self, Write},
    process::{Command, Stdio},
};

use crate::{cli::InputType, Opts};

pub(crate) fn resolve_username(opts: &Opts) -> Result<String> {
    Ok(
        if let Some(user) = resolve_cred(&opts.username, opts.username_type) {
            user
        } else {
            print!("Username: ");
            io::stdout()
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
        prompt_password("Password: ").context("Failed to prompt for password")
    }
}

fn resolve_cred(cred: &Option<String>, t: InputType) -> Option<String> {
    cred.as_ref().and_then(|input| match t {
        InputType::Text => Some(input.into()),
        InputType::File => read_to_string(input)
            .map_err(|e| println!("Warning: Failed to read {input}:\n{e}"))
            .ok(),
        InputType::Command => Command::new(
            #[cfg(unix)]
            "sh",
            #[cfg(windows)]
            "cmd.exe",
        )
        .arg(
            #[cfg(unix)]
            "-c",
            #[cfg(windows)]
            "/c",
        )
        .arg(input)
        .stderr(Stdio::inherit())
        .stdin(Stdio::inherit())
        .output()
        .map_err(|e| println!("Warning: Failed to execute command: {e}"))
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout)
                    .map_err(|e| println!("Warning: {e}"))
                    .ok()
            } else {
                println!("Warning: command exited with {}", output.status);
                None
            }
        }),
    })
}
