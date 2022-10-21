use anyhow::{Context, Result};
use rpassword::read_password;

use std::{
    ffi::OsString,
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

fn resolve_cred(cred: &Option<OsString>, t: InputType) -> Option<String> {
    let input = cred.as_ref()?;
    if input.is_empty() {
        return None;
    }

    match t {
        InputType::Text => input
            .clone()
            .into_string()
            .map_err(|_| eprintln!("Warning: invalid UTF-8"))
            .ok(),
        InputType::File => read_to_string(input)
            .map_err(|e| eprintln!("Warning: Failed to read {}:\n{e}", input.to_string_lossy()))
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

#[cfg(test)]
mod tests {
    use tempfile::NamedTempFile;

    use std::io::Write;

    use crate::cli::InputType;

    use super::resolve_cred;

    #[test]
    fn resolve_cred_none() {
        assert_eq!(resolve_cred(&None, InputType::Text), None);
        assert_eq!(resolve_cred(&Some("".into()), InputType::Text), None);
    }

    #[test]
    fn resolve_cred_text() {
        assert_eq!(
            resolve_cred(&Some("foo".into()), InputType::Text),
            Some("foo".into())
        );
    }

    #[test]
    fn resolve_cred_file() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "foo").unwrap();
        assert_eq!(
            resolve_cred(&Some(file.path().into()), InputType::File),
            Some("foo".into())
        );
    }

    #[test]
    fn resolve_cred_command() {
        assert!(resolve_cred(&Some("echo foo".into()), InputType::Command)
            .unwrap()
            .starts_with("foo"))
    }
}
