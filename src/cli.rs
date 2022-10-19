use clap::{Parser, ValueEnum};

use std::path::PathBuf;

/// A command-line submission tool for the UMD CS Submission Server
/// https://github.com/figsoda/sagoin
#[derive(Parser)]
#[command(verbatim_doc_comment, version)]
pub struct Opts {
    /// Set the working directory
    #[arg(value_name = "directory")]
    pub dir: Option<PathBuf>,

    /// Don't submit the project
    #[arg(short, long)]
    pub no_submit: bool,

    /// Open project page
    #[arg(short, long)]
    pub open: bool,

    /// Specify the username for authentication, see --username-type for more information
    #[arg(short, long, env = "SAGOIN_USERNAME", value_name = "username")]
    pub(crate) username: Option<String>,

    /// Specify the type for the username
    #[arg(
        short = 'U',
        long,
        env = "SAGOIN_USERNAME_TYPE",
        value_name = "type",
        default_value = "text"
    )]
    pub(crate) username_type: InputType,

    /// Specify the password for authentication, see --password-type for more information
    #[arg(short, long, env = "SAGOIN_PASSWORD", value_name = "password")]
    pub(crate) password: Option<String>,

    /// Specify the type for the password
    #[arg(
        short = 'P',
        long,
        env = "SAGOIN_PASSWORD_TYPE",
        value_name = "type",
        default_value = "text"
    )]
    pub(crate) password_type: InputType,

    /// Command to run before submission
    #[arg(
        short = 's',
        long,
        env = "SAGOIN_PRE_SUBMIT_HOOK",
        value_name = "command"
    )]
    pub(crate) pre_submit_hook: Option<String>,

    /// Command to run after successful submissions
    #[arg(
        short = 'S',
        long,
        env = "SAGOIN_POST_SUBMIT_HOOK",
        value_name = "command"
    )]
    pub(crate) post_submit_hook: Option<String>,
}

#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum InputType {
    Command,
    File,
    Text,
}
