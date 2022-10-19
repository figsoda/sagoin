#![forbid(unsafe_code)]

use anyhow::{Context, Result};
use clap::Parser;
use ignore::WalkBuilder;
use is_executable::IsExecutable;
use zip::{write::FileOptions, ZipWriter};

use std::{
    env::set_current_dir,
    ffi::OsStr,
    fs::File,
    io::{self, Cursor, Seek},
};

use sagoin::{cli::Opts, get_course_url, submit};

fn main() -> Result<()> {
    let opts = Opts::parse();

    if let Some(dir) = &opts.dir {
        set_current_dir(dir).context("Failed to set current dir")?;
    }

    let props = java_properties::read(File::open(".submit").context("Failed to read .submit")?)
        .context("Failed to parse .submit")?;

    if !opts.no_submit {
        let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
        zip.set_comment("");
        let regular = FileOptions::default();
        let executable = regular.unix_permissions(0o755);
        for entry in WalkBuilder::new(".").hidden(false).build() {
            let entry = entry.context("Failed to read entry")?;
            let path = entry.into_path();
            let path = path.strip_prefix(".")?;
            if !path.is_file() || path.file_name() == Some(OsStr::new(".submitUser")) {
                continue;
            }

            zip.start_file(
                path.to_string_lossy(),
                if path.is_executable() {
                    executable
                } else {
                    regular
                },
            )
            .context("Failed to write to the zip file")?;

            io::copy(
                &mut File::open(path).context(format!("Failed to read {}", path.display()))?,
                &mut zip,
            )
            .context("Failed to write to the zip file")?;
        }

        let mut zip = zip
            .finish()
            .context("Failed to finish writing to the zip file")?;
        zip.rewind()
            .context("Failed to rewind to the beginning of the zip file")?;

        submit(
            File::open(".submitUser")
                .ok()
                .and_then(|file| java_properties::read(file).ok())
                .unwrap_or_default(),
            &props,
            &opts,
            zip,
        )?;
    }

    if opts.open {
        webbrowser::open(&get_course_url(&props)?).context("Failed to open the web browser")?;
    }

    Ok(())
}
