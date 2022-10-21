use anyhow::{Context, Result};
use rpassword::read_password;

use std::{
    ffi::OsString,
    fs::read_to_string,
    io::{self, Write},
    process::{Output, Stdio},
};

use crate::{cli::InputType, cmd, state::State, warn, Opts};

impl<W: Write> State<W> {
    pub(crate) fn resolve_username(&mut self, opts: &Opts) -> Result<String> {
        Ok(
            if let Some(user) = self.resolve_cred(&opts.username, opts.username_type) {
                user
            } else {
                self.prompt("Username")?;

                let mut user = String::new();
                io::stdin()
                    .read_line(&mut user)
                    .context("Failed to prompt for username")?;

                user
            },
        )
    }

    pub(crate) fn resolve_password(&mut self, opts: &Opts) -> Result<String> {
        if let Some(pass) = self.resolve_cred(&opts.password, opts.password_type) {
            Ok(pass)
        } else {
            self.prompt("Password")?;
            read_password().context("Failed to prompt for password")
        }
    }

    fn resolve_cred(&mut self, cred: &Option<OsString>, t: InputType) -> Option<String> {
        let input = cred.as_ref()?;
        if input.is_empty() {
            return None;
        }

        match t {
            InputType::Text => input
                .clone()
                .into_string()
                .map_err(|_| warn!(self, "invalid UTF-8"))
                .ok(),
            InputType::File => read_to_string(input)
                .map_err(|e| warn!(self, "failed to read {}:\n{e}", input.to_string_lossy()))
                .ok(),
            InputType::Command => cmd::shell()
                .arg(input)
                .stderr(Stdio::inherit())
                .stdin(Stdio::inherit())
                .output()
                .map_err(|e| warn!(self, "failed to execute command: {e}"))
                .ok()
                .and_then(|Output { status, stdout, .. }| {
                    if status.success() {
                        String::from_utf8(stdout)
                            .map_err(|e| warn!(self, "{e}"))
                            .ok()
                    } else {
                        warn!(self, "command failed with exit code {status}");
                        None
                    }
                }),
        }
    }
}

#[cfg(test)]
mod tests {
    use tempfile::NamedTempFile;

    use std::io::Write;

    use crate::{cli::InputType, state::State};

    #[test]
    fn resolve_cred_none() {
        let mut state = State::sink();
        assert_eq!(state.resolve_cred(&None, InputType::Text), None);
        assert_eq!(state.resolve_cred(&Some("".into()), InputType::Text), None);
    }

    #[test]
    fn resolve_cred_command() {
        assert!(State::sink()
            .resolve_cred(&Some("echo foo".into()), InputType::Command)
            .unwrap()
            .starts_with("foo"))
    }

    #[test]
    fn resolve_cred_file() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "foo").unwrap();
        assert_eq!(
            State::sink().resolve_cred(&Some(file.path().into()), InputType::File),
            Some("foo".into())
        );
    }

    #[test]
    fn resolve_cred_text() {
        assert_eq!(
            State::sink().resolve_cred(&Some("foo".into()), InputType::Text),
            Some("foo".into())
        );
    }
}
