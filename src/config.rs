use clap::Parser;
use eyre::{Result, WrapErr};
use serde::Deserialize;

use std::{
    ffi::OsString,
    fs,
    io::{StderrLock, Write},
    path::PathBuf,
};

use crate::{
    cli::{InputType, Opts},
    state::State,
    warn,
};

pub struct Config {
    pub dir: Option<PathBuf>,
    pub no_submit: bool,
    pub list_files: bool,
    pub info: bool,
    pub open: bool,
    pub time_format: String,
    pub(crate) username: Option<Credential>,
    pub(crate) password: Option<Credential>,
    pub(crate) pre_submit_hook: Option<OsString>,
    pub(crate) post_submit_hook: Option<OsString>,
}

pub(crate) enum Credential {
    Command(OsString),
    File(OsString),
    Text(String),
}

#[derive(Deserialize)]
struct ConfigFile {
    time_format: Option<String>,
    username: Option<String>,
    username_type: Option<InputType>,
    password: Option<String>,
    password_type: Option<InputType>,
    pre_submit_hook: Option<OsString>,
    post_submit_hook: Option<OsString>,
}

pub fn load_config() -> Result<(Config, State<StderrLock<'static>>)> {
    let opts = Opts::parse();
    opts.color.apply();

    let mut state = State::stderr()?;

    Ok((
        if let Some(path) = opts.config.or_else(find_config_file) {
            let cfg: ConfigFile = toml::from_slice(
                &fs::read(&path).wrap_err_with(|| format!("failed to read {}", path.display()))?,
            )?;

            Config {
                dir: opts.dir,
                no_submit: opts.no_submit,
                list_files: opts.list_files,
                info: opts.info,
                open: opts.open,
                time_format: opts
                    .time_format
                    .or(cfg.time_format)
                    .unwrap_or_else(default_time_format),
                username: Credential::from_fallback(
                    &mut state,
                    "username",
                    opts.username,
                    cfg.username,
                    opts.username_type.or(cfg.username_type),
                ),
                password: Credential::from_fallback(
                    &mut state,
                    "password",
                    opts.password,
                    cfg.password,
                    opts.password_type.or(cfg.password_type),
                ),
                pre_submit_hook: opts.pre_submit_hook.or(cfg.pre_submit_hook),
                post_submit_hook: opts.post_submit_hook.or(cfg.post_submit_hook),
            }
        } else {
            Config {
                dir: opts.dir,
                no_submit: opts.no_submit,
                list_files: opts.list_files,
                info: opts.info,
                open: opts.open,
                time_format: opts.time_format.unwrap_or_else(default_time_format),
                username: opts.username.and_then(|user| {
                    Credential::from_os_string(&mut state, "username", user, opts.username_type)
                }),
                password: opts.password.and_then(|pass| {
                    Credential::from_os_string(&mut state, "password", pass, opts.password_type)
                }),
                pre_submit_hook: opts.pre_submit_hook,
                post_submit_hook: opts.post_submit_hook,
            }
        },
        state,
    ))
}

fn default_time_format() -> String {
    "[month repr:short] [day padding:none], [hour]:[minute]".into()
}

impl Credential {
    fn from_fallback(
        state: &mut State<impl Write>,
        name: &'static str,
        x: Option<OsString>,
        y: Option<String>,
        t: Option<InputType>,
    ) -> Option<Self> {
        if let Some(input) = x {
            Self::from_os_string(state, name, input, t)
        } else if let Some(input) = y {
            Self::from_string(input, t)
        } else {
            None
        }
    }

    fn from_os_string(
        state: &mut State<impl Write>,
        name: &'static str,
        input: OsString,
        t: Option<InputType>,
    ) -> Option<Self> {
        if input.is_empty() {
            None
        } else {
            match t.unwrap_or(InputType::Text) {
                InputType::Command => Some(Self::Command(input)),
                InputType::File => Some(Self::File(input)),
                InputType::Text => input
                    .into_string()
                    .map_err(|_| warn!(state, "{name} contains invalid UTF-8"))
                    .ok()
                    .map(Self::Text),
            }
        }
    }

    fn from_string(input: String, t: Option<InputType>) -> Option<Self> {
        (!input.is_empty()).then(|| match t.unwrap_or(InputType::Text) {
            InputType::Command => Self::Command(input.into()),
            InputType::File => Self::File(input.into()),
            InputType::Text => Self::Text(input),
        })
    }
}

#[cfg(unix)]
fn find_config_file() -> Option<PathBuf> {
    xdg::BaseDirectories::with_prefix("sagoin")
        .ok()
        .and_then(|dirs| dirs.find_config_file("config.toml"))
}

#[cfg(windows)]
fn find_config_file() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("sagoin").join("config.toml"))
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use crate::{cli::InputType, state::State};

    use super::Credential;

    #[test]
    fn credential_from_fallback_none() {
        let mut state = State::buffer();
        assert!(Credential::from_fallback(&mut state, "password", None, None, None).is_none());
        assert!(state.out.is_empty());
    }

    #[test]
    #[cfg(unix)]
    fn credential_from_fallback_invalid() {
        use std::os::unix::ffi::OsStringExt;

        let mut state = State::buffer();
        assert!(Credential::from_fallback(
            &mut state,
            "password",
            Some(OsString::from_vec(vec![0xff])),
            None,
            None
        )
        .is_none());
        assert!(!state.out.is_empty());
    }

    #[test]
    #[cfg(windows)]
    fn credential_from_fallback_invalid() {
        use std::os::windows::ffi::OsStringExt;

        let mut state = State::buffer();
        assert!(Credential::from_fallback(
            &mut state,
            "password",
            Some(OsString::from_wide(&[0xdfff])),
            None,
            None
        )
        .is_none());
        assert!(!state.out.is_empty());
    }

    #[test]
    fn credential_from_fallback_config_file() {
        let mut state = State::buffer();
        assert!(matches!(
            Credential::from_fallback(
                &mut state,
                "password",
                None,
                Some("foo".into()),
                Some(InputType::File),
            ),
            Some(Credential::File(input)) if input == "foo"
        ));
        assert!(state.out.is_empty());
    }

    #[test]
    fn credential_from_fallback_both() {
        let mut state = State::buffer();
        assert!(matches!(
            Credential::from_fallback(
                &mut state,
                "password",
                Some("foo".into()),
                Some("bar".into()),
                None,
            ),
            Some(Credential::Text(input)) if input == "foo"
        ));
        assert!(state.out.is_empty());
    }
}
