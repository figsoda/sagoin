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
    pub open: bool,
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
pub struct ConfigFile {
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
        if let Some(path) = find_config_file() {
            let cfg: ConfigFile = toml::from_slice(
                &fs::read(&path).wrap_err_with(|| format!("failed to read {}", path.display()))?,
            )?;

            Config {
                dir: opts.dir,
                no_submit: opts.no_submit,
                open: opts.open,
                username: Credential::from_fallback(
                    opts.username,
                    cfg.username,
                    opts.username_type.or(cfg.username_type),
                )
                .and_then(|user| {
                    if user.is_none() {
                        warn!(state, "username contains ivalid UTF-8");
                    }
                    user
                }),
                password: Credential::from_fallback(
                    opts.password,
                    cfg.password,
                    opts.password_type.or(cfg.password_type),
                )
                .and_then(|pass| {
                    if pass.is_none() {
                        warn!(state, "password contains ivalid UTF-8");
                    }
                    pass
                }),
                pre_submit_hook: opts.pre_submit_hook.or(cfg.pre_submit_hook),
                post_submit_hook: opts.post_submit_hook.or(cfg.post_submit_hook),
            }
        } else {
            Config {
                dir: opts.dir,
                no_submit: opts.no_submit,
                open: opts.open,
                username: opts.username.and_then(|user| {
                    if user.is_empty() {
                        None
                    } else {
                        Credential::from_os_string(user, opts.username_type).or_else(|| {
                            warn!(state, "username contains invalid UTF8");
                            None
                        })
                    }
                }),
                password: opts.password.and_then(|pass| {
                    if pass.is_empty() {
                        None
                    } else {
                        Credential::from_os_string(pass, opts.password_type).or_else(|| {
                            warn!(state, "password contains invalid UTF8");
                            None
                        })
                    }
                }),
                pre_submit_hook: opts.pre_submit_hook,
                post_submit_hook: opts.post_submit_hook,
            }
        },
        state,
    ))
}

impl Credential {
    fn from_fallback(
        x: Option<OsString>,
        y: Option<String>,
        t: Option<InputType>,
    ) -> Option<Option<Self>> {
        if let Some(input) = x {
            (!input.is_empty()).then(|| Self::from_os_string(input, t))
        } else if let Some(input) = y {
            (!input.is_empty()).then(|| Some(Self::from_string(input, t)))
        } else {
            None
        }
    }

    fn from_os_string(input: OsString, t: Option<InputType>) -> Option<Self> {
        match t.unwrap_or(InputType::Text) {
            InputType::Command => Some(Self::Command(input)),
            InputType::File => Some(Self::File(input)),
            InputType::Text => input.into_string().ok().map(Self::Text),
        }
    }

    fn from_string(input: String, t: Option<InputType>) -> Self {
        match t.unwrap_or(InputType::Text) {
            InputType::Command => Self::Command(input.into()),
            InputType::File => Self::File(input.into()),
            InputType::Text => Self::Text(input),
        }
    }
}

#[cfg(unix)]
fn find_config_file() -> Option<PathBuf> {
    xdg::BaseDirectories::with_prefix("sagoin")
        .ok()
        .and_then(|dirs| dirs.find_config_file("config.toml"))
}

#[cfg(not(unix))]
fn find_config() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("sagoin").join("config.toml"))
}
