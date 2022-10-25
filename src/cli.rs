use clap::{Parser, ValueEnum};
use concolor_clap::{color_choice, Color};

use std::{ffi::OsString, path::PathBuf};

/// A command-line submission tool for the UMD CS Submission Server
/// https://github.com/figsoda/sagoin
#[derive(Parser)]
#[command(color = color_choice(), version, verbatim_doc_comment)]
pub(crate) struct Opts {
    /// Set the working directory, all commands will be run under this directory
    #[arg(value_name = "DIRECTORY")]
    pub dir: Option<PathBuf>,

    /// Don't submit the project
    #[arg(short, long)]
    pub no_submit: bool,

    /// Show information about the project and exit
    #[arg(short, long)]
    pub info: bool,

    /// Open the project page in a web browser
    #[arg(short, long)]
    pub open: bool,

    #[command(flatten)]
    pub color: Color,

    /// Specify the path to the config file,
    #[cfg_attr(
        unix,
        doc = "looks for sagoin/config.toml under XDG configuration directories"
    )]
    #[cfg_attr(
        windows,
        doc = "defaults to {FOLDERID_RoamingAppData}\\sagoin\\config.toml"
    )]
    /// when unspecified
    #[arg(short, long, env = "SAGOIN_CONFIG", value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Specify how to format the due date, ignored without the --info flag,
    /// defaults to "[month repr:short] [day padding:none], [hour]:[minute]" when unspecified
    ///
    /// See https://time-rs.github.io/book/api/format-description.html for more information
    #[arg(short, long, env = "SAGOIN_TIME_FORMAT", value_name = "FORMAT")]
    pub time_format: Option<String>,

    /// Specify the username for authentication,
    /// see --username-type for more information
    #[arg(short, long, env = "SAGOIN_USERNAME")]
    pub username: Option<OsString>,

    /// Specify the type for the username, defaults to text when unspecified
    ///
    /// text: the specified username will be used as is
    /// file: the username will be read from the specified file
    /// command: the specified command will be run in a shell and the stdout will be used as the username if successful
    #[arg(
        short = 'U',
        long,
        env = "SAGOIN_USERNAME_TYPE",
        value_name = "TYPE",
        verbatim_doc_comment
    )]
    pub username_type: Option<InputType>,

    /// Specify the password for authentication,
    /// see --password-type for more information
    #[arg(short, long, env = "SAGOIN_PASSWORD")]
    pub password: Option<OsString>,

    /// Specify the type for the password, defaults to text when unspecified
    ///
    /// text: the specified password will be used as is
    /// file: the password will be read from the specified file
    /// command: the specified command will be run in a shell and the stdout will be used as the password if successful
    #[arg(
        short = 'P',
        long,
        env = "SAGOIN_PASSWORD_TYPE",
        value_name = "TYPE",
        verbatim_doc_comment
    )]
    pub password_type: Option<InputType>,

    /// Command to run before submission
    ///
    /// You can do things like running tests, checking for code styles, and running git pre-commit hooks
    /// Submission will be aborted if the command fails
    #[arg(
        short = 's',
        long,
        env = "SAGOIN_PRE_SUBMIT_HOOK",
        value_name = "COMMAND",
        verbatim_doc_comment
    )]
    pub pre_submit_hook: Option<OsString>,

    /// Command to run after successful submissions
    ///
    /// You can do things like sending notifications and git pushing
    /// Submission will NOT be aborted if the command fails
    #[arg(
        short = 'S',
        long,
        env = "SAGOIN_POST_SUBMIT_HOOK",
        value_name = "COMMAND",
        verbatim_doc_comment
    )]
    pub post_submit_hook: Option<OsString>,
}

#[derive(Clone, Copy, ValueEnum)]
#[cfg_attr(
    not_build,
    derive(serde::Deserialize),
    serde(rename_all = "kebab-case")
)]
pub(crate) enum InputType {
    Command,
    File,
    Text,
}
