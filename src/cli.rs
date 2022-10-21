use clap::{Parser, ValueEnum};

use std::{ffi::OsString, path::PathBuf};

/// A command-line submission tool for the UMD CS Submission Server
/// https://github.com/figsoda/sagoin
#[derive(Parser)]
#[command(version, verbatim_doc_comment)]
pub struct Opts {
    /// Set the working directory
    #[arg(value_name = "directory")]
    pub dir: Option<PathBuf>,

    /// Don't submit the project
    #[arg(short, long)]
    pub no_submit: bool,

    /// Open the project page in a web browser
    #[arg(short, long)]
    pub open: bool,

    /// Specify the username for authentication,
    /// see --username-type for more information
    #[arg(short, long, env = "SAGOIN_USERNAME", value_name = "username")]
    pub(crate) username: Option<OsString>,

    /// Specify the type for the username
    ///
    /// text: the specified username will be used as is
    /// file: the username will be read from the specified file
    /// command: the specified command will be run in a shell and the stdout will be used as the username if successful
    #[arg(
        short = 'U',
        long,
        env = "SAGOIN_USERNAME_TYPE",
        value_name = "type",
        default_value = "text",
        verbatim_doc_comment
    )]
    pub(crate) username_type: InputType,

    /// Specify the password for authentication,
    /// see --password-type for more information
    #[arg(short, long, env = "SAGOIN_PASSWORD", value_name = "password")]
    pub(crate) password: Option<OsString>,

    /// Specify the type for the password
    ///
    /// text: the specified password will be used as is
    /// file: the password will be read from the specified file
    /// command: the specified command will be run in a shell and the stdout will be used as the password if successful
    #[arg(
        short = 'P',
        long,
        env = "SAGOIN_PASSWORD_TYPE",
        value_name = "type",
        default_value = "text",
        verbatim_doc_comment
    )]
    pub(crate) password_type: InputType,

    /// Command to run before submission
    ///
    /// You can do things like running tests, checking for code styles, and running git pre-commit hooks
    /// Submission will be aborted if the command fails
    #[arg(
        short = 's',
        long,
        env = "SAGOIN_PRE_SUBMIT_HOOK",
        value_name = "command",
        verbatim_doc_comment
    )]
    pub(crate) pre_submit_hook: Option<OsString>,

    /// Command to run after successful submissions
    ///
    /// You can do things like sending notifications and git pushing
    /// Submission will NOT be aborted if the command fails
    #[arg(
        short = 'S',
        long,
        env = "SAGOIN_POST_SUBMIT_HOOK",
        value_name = "command",
        verbatim_doc_comment
    )]
    pub(crate) post_submit_hook: Option<OsString>,
}

#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum InputType {
    Command,
    File,
    Text,
}
