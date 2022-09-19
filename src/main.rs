#![forbid(unsafe_code)]

mod auth;
mod cli;
mod props;

use anyhow::{anyhow, Context};
use clap::Parser;
use ignore::WalkBuilder;
use is_executable::IsExecutable;
use zip::{write::FileOptions, ZipWriter};

use std::{
    env::set_current_dir,
    fs::File,
    io::{self, Cursor, Seek},
};

use crate::{
    auth::negotiate_otp,
    cli::Opts,
    props::{read_submit, read_submit_user, Props},
};

fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();

    if let Some(dir) = opts.dir {
        set_current_dir(&dir).context("Failed to set current dir")?;
    }

    let mut props = read_submit()?;
    if let Ok(submit_user) = File::open(".submitUser") {
        read_submit_user(&mut props, submit_user);
    }

    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    zip.set_comment("");
    let regular = FileOptions::default();
    let executable = regular.unix_permissions(0o755);
    for entry in WalkBuilder::new(".").hidden(false).build() {
        let entry = entry.context("Failed to read entry")?;
        let path = entry.into_path();
        if !path.is_file() {
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
            &mut File::open(&path).context(format!("Failed to read {}", path.display()))?,
            &mut zip,
        )
        .context("Failed to write to the zip file")?;
    }

    let mut zip = zip
        .finish()
        .context("Failed to finish writing to the zip file")?;
    zip.rewind()
        .context("Failed to rewind to the beginning of the zip file")?;

    submit(props, zip, true)
}

fn submit(props: Props, zip: Cursor<Vec<u8>>, reauth: bool) -> anyhow::Result<()> {
    let mut props = props;

    if reauth && (props.cvs.is_none() && props.class.is_none() || props.otp.is_none()) {
        return submit(negotiate_otp(props)?, zip, false);
    }

    let parts = props
        .parts
        .add_text("submitClientTool", "sagoin")
        .add_text("submitClientVersion", env!("CARGO_PKG_VERSION"))
        .add_stream(
            "submittedFiles",
            zip.clone(),
            Some("submit.zip"),
            Some(
                "application/x-zip-compressed"
                    .parse()
                    .context("Failed to parse the mime type for the zip")?,
            ),
        )
        .prepare()?;

    match ureq::post(props.url.as_ref().context("submitURL is null in .submit")?)
        .set(
            "Content-Type",
            &format!("multipart/form-data; boundary={}", parts.boundary()),
        )
        .send(parts)
    {
        Ok(resp) => {
            if let Ok(success) = resp.into_string() {
                print!("{}", success);
            } else {
                println!("Successfull submission received");
            }

            Ok(())
        }
        Err(ureq::Error::Status(500, resp)) => {
            println!("Warning: Status code 500");
            if let Ok(err) = resp.into_string() {
                print!("Warning: {}", err);
            }
            submit(negotiate_otp(props)?, zip, false)
        }
        Err(ureq::Error::Status(code, resp)) => Err(if let Ok(err) = resp.into_string() {
            anyhow!("{}", err.trim_end())
                .context(format!("Status code {code}"))
                .context("Failed to submit project")
        } else {
            anyhow!("Status code {code}").context("Failed to submit project")
        }),
        e => e
            .map(|_| ())
            .context("Failed to send request to the submit server"),
    }
}
