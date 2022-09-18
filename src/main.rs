#![forbid(unsafe_code)]

mod cli;
mod types;

use anyhow::{anyhow, Context};
use clap::Parser;
use ignore::WalkBuilder;
use is_executable::IsExecutable;
use java_properties::PropertiesIter;
use reqwest::multipart::Part;
use zip::{write::FileOptions, ZipWriter};

use std::{
    env::set_current_dir,
    fs::File,
    io::{self, Cursor},
    mem,
};

use crate::{cli::Opts, types::Submit};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();

    if let Some(dir) = opts.dir {
        set_current_dir(&dir).context("Failed to set current dir")?;
    }

    let mut submit = Submit::default();

    PropertiesIter::new(File::open(".submit").context("Failed to read .submit")?)
        .read_into(|k, v| {
            match k.as_ref() {
                "authentication.type" => submit.auth = Some(v.clone()),
                "baseURL" => submit.base_url = Some(v.clone()),
                "courseKey" => submit.course_key = Some(v.clone()),
                "projectNumber" => submit.project = Some(v.clone()),
                "submitURL" => {
                    submit.url = Some(v);
                    return;
                }
                _ => {}
            };
            submit.form = mem::take(&mut submit.form).text(k, v);
        })
        .context("Failed to parse .submit")?;

    if let Ok(submit_user) = File::open(".submitUser") {
        if let Err(e) = PropertiesIter::new(submit_user).read_into(|k, v| {
            match k.as_ref() {
                "classAccount" => submit.class = Some(v.clone()),
                "cvsAccount" => submit.cvs = Some(v.clone()),
                "loginName" => submit.login = Some(v.clone()),
                "oneTimePassword" => submit.otp = Some(v.clone()),
                _ => {}
            }
            submit.form = mem::take(&mut submit.form).text(k, v);
        }) {
            eprintln!("Warning: error when parsing .submitUser: {e}");
        }
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

    let cl = reqwest::Client::new();

    let resp = cl
        .post(submit.url.context("submitURL is null in .submit")?)
        .multipart(
            submit
                .form
                .text("submitClientTool", "sagoin")
                .text("submitClientVersion", env!("CARGO_PKG_VERSION"))
                .part(
                    "submittedFiles",
                    Part::bytes(
                        zip.finish()
                            .context("Failed to finish writing to the zip file")?
                            .into_inner(),
                    )
                    .file_name("submit.zip"),
                ),
        )
        .send()
        .await
        .context("Failed to send request to the submit server")?;

    let status = resp.status();
    if status.is_success() {
        if let Ok(success) = resp.text().await {
            print!("{}", success);
        } else {
            println!("Successfull submission received");
        }

        Ok(())
    } else {
        Err(if let Ok(err) = resp.text().await {
            anyhow!("{}", err.trim_end())
                .context(status)
                .context("Failed to submit project")
        } else {
            anyhow!(status).context("Failed to submit project")
        })
    }
}
