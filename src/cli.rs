use clap::{Parser, ValueEnum};

use std::path::PathBuf;

#[derive(Parser)]
#[command(version)]
pub struct Opts {
    #[arg(value_name = "directory")]
    pub dir: Option<PathBuf>,

    #[arg(short, long)]
    pub no_submit: bool,

    #[arg(short, long)]
    pub open: bool,

    #[arg(short, long, env = "SAGOIN_USERNAME", value_name = "username")]
    pub username: Option<String>,

    #[arg(
        short = 'U',
        long,
        env = "SAGOIN_USERNAME_TYPE",
        value_name = "type",
        default_value = "text"
    )]
    pub username_type: InputType,

    #[arg(short, long, env = "SAGOIN_PASSWORD", value_name = "password")]
    pub password: Option<String>,

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
