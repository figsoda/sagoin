use clap::Parser;

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
}
