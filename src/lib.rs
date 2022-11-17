//! > **NOTE: using `sagoin` as a library is not supported**
//!
//!
//! `sagoin` is a command-line submission tool for the UMD CS Submit Server.
//!
//! ```sh
//! cargo install sagoin
//! ```
//!
//! Check out the [GitHub repository](https://github.com/figsoda/sagoin) for more information

#![forbid(unsafe_code)]

mod auth;
mod cli;
mod cmd;
pub mod config;
pub mod course;
mod cred;
pub mod state;
mod submit;

use eyre::{eyre, Result};

use std::collections::HashMap;

type Props = HashMap<String, String>;

trait PropsExt {
    fn get_prop(&self, key: &'static str) -> Result<&String>;
}

impl PropsExt for Props {
    fn get_prop(&self, key: &'static str) -> Result<&String> {
        self.get(key)
            .ok_or_else(|| eyre!("{key} is null in .submit"))
    }
}
