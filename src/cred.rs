use eyre::{Result, WrapErr};
use rpassword::read_password;

use std::{
    fs::read_to_string,
    io::{self, Write},
    process::{Output, Stdio},
};

use crate::{cmd, config::Credential, state::State, warn};

impl<W: Write> State<W> {
    pub(crate) fn resolve_username(&mut self, user: &Option<Credential>) -> Result<String> {
        Ok(if let Some(user) = self.resolve_cred(user) {
            user
        } else {
            self.prompt("Username")?;

            let mut user = String::new();
            io::stdin()
                .read_line(&mut user)
                .wrap_err("failed to prompt for username")?;

            user
        })
    }

    pub(crate) fn resolve_password(&mut self, pass: &Option<Credential>) -> Result<String> {
        if let Some(pass) = self.resolve_cred(pass) {
            Ok(pass)
        } else {
            self.prompt("Password")?;
            read_password().wrap_err("failed to prompt for password")
        }
    }

    fn resolve_cred(&mut self, cred: &Option<Credential>) -> Option<String> {
        match cred.as_ref()? {
            Credential::Text(input) => Some(input.clone()),
            Credential::File(input) => read_to_string(input)
                .map_err(|e| warn!(self, "failed to read {}:\n{e}", input.to_string_lossy()))
                .ok(),
            Credential::Command(input) => cmd::shell()
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

    use crate::{config::Credential, state::State};

    #[test]
    fn resolve_cred_none() {
        let mut state = State::sink();
        assert_eq!(state.resolve_cred(&None), None);
    }

    #[test]
    fn resolve_cred_command() {
        assert!(State::sink()
            .resolve_cred(&Some(Credential::Command("echo foo".into())))
            .unwrap()
            .starts_with("foo"));
    }

    #[test]
    fn resolve_cred_file() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "foo").unwrap();
        assert_eq!(
            State::sink().resolve_cred(&Some(Credential::File(file.path().into()))),
            Some("foo".into()),
        );
    }

    #[test]
    fn resolve_cred_text() {
        assert_eq!(
            State::sink().resolve_cred(&Some(Credential::Text("foo".into()))),
            Some("foo".into()),
        );
    }
}
