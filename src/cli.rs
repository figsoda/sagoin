use clap::Parser;

use std::path::PathBuf;

#[derive(Parser)]
#[clap(version)]
pub struct Opts {
    #[clap(value_name = "directory")]
    pub dir: Option<PathBuf>,
}
