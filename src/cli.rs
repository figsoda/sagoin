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
    pub username: Option<String>,

    /// Specify the type for the username
    #[arg(
        short = 'U',
        long,
        env = "SAGOIN_USERNAME_TYPE",
        value_name = "type",
        default_value = "text"
    )]
    pub username_type: InputType,

    /// Specify the password for authentication, see --password-type for more information
    #[arg(short, long, env = "SAGOIN_PASSWORD", value_name = "password")]
    pub password: Option<String>,

    /// Specify the type for the password
    #[arg(
        short = 'P',
        long,
        env = "SAGOIN_PASSWORD_TYPE",
        value_name = "type",
        default_value = "text"
    )]
    pub password_type: InputType,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum InputType {
    Command,
    File,
    Text,
}
