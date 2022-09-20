#![forbid(unsafe_code)]

mod auth;
mod cli;

use anyhow::{anyhow, Context};
use clap::Parser;
use icalendar::parser::{read_calendar_simple, unfold};
use ignore::WalkBuilder;
use is_executable::IsExecutable;
use multipart::client::lazy::Multipart;
use zip::{write::FileOptions, ZipWriter};

use std::{
    collections::HashMap,
    env::set_current_dir,
    fs::File,
    io::{self, Cursor, Seek},
};

use crate::{auth::negotiate_otp, cli::Opts};

fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();

    if let Some(dir) = opts.dir {
        set_current_dir(&dir).context("Failed to set current dir")?;
    }

    let props = java_properties::read(File::open(".submit").context("Failed to read .submit")?)
        .context("Failed to parse .submit")?;

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

    submit(
        File::open(".submitUser")
            .ok()
            .and_then(|file| java_properties::read(file).ok())
            .unwrap_or_default(),
        &props,
        zip,
        true,
    )?;

    if opts.open {
        webbrowser::open(&get_course_url(&props)?).context("Failed to open the web browser")?;
    }

    Ok(())
}

fn submit(
    user_props: HashMap<String, String>,
    props: &HashMap<String, String>,
    zip: Cursor<Vec<u8>>,
    reauth: bool,
) -> anyhow::Result<()> {
    if reauth
        && (!user_props.contains_key("cvsAccount") && !user_props.contains_key("classAccount")
            || !user_props.contains_key("oneTimePassword"))
    {
        return submit(negotiate_otp(props)?, props, zip, false);
    }

    let mut parts = Multipart::new();

    for (k, v) in user_props.iter().chain(props) {
        parts.add_text(k.to_owned(), v);
    }

    let parts = parts
        .add_text("submitClientTool", "sagoin")
        .add_text("submitClientVersion", env!("CARGO_PKG_VERSION"))
        .add_stream(
            "submittedFiles",
            zip.clone(),
            Some("submit.zip"),
            Some(
                "application/zip"
                    .parse()
                    .context("Failed to parse application/zip as a mime type")?,
            ),
        )
        .prepare()?;

    match ureq::post(
        props
            .get("submitURL")
            .context("submitURL is null in .submit")?,
    )
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
            submit(negotiate_otp(props)?, props, zip, false)
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

fn get_course_url(props: &HashMap<String, String>) -> anyhow::Result<String> {
    let proj = format!(
        "{} project {}: ",
        props
            .get("courseName")
            .context("courseName is null in .submit")?,
        props
            .get("projectNumber")
            .context("projectNumber is null in .submit")?
    );

    read_calendar_simple(&unfold(
        &ureq::get(&format!(
            "{}/feed/CourseCalendar?courseKey={}",
            props.get("baseURL").context("baseURL is null in .submit")?,
            props
                .get("courseKey")
                .context("courseKey is null in .submit")?,
        ))
        .call()
        .context("Failed to download the course calendar")?
        .into_string()
        .context("Failed to parse the course calendar")?,
    ))
    .map_err(|e| anyhow!("{}", e).context("Failed to parse the course calendar"))?
    .get(0)
    .and_then(|root| {
        root.components.iter().find_map(|component| {
            let mut url = None;
            let mut found = false;

            for prop in component.properties.iter() {
                match prop.name.as_str() {
                    "SUMMARY" => {
                        if prop.val.as_str().starts_with(&proj) {
                            found = true;
                        } else {
                            return None;
                        }
                    }

                    "URL" => url = Some(prop.val.to_string()),
                    _ => {}
                }
            }

            if found {
                url
            } else {
                None
            }
        })
    })
    .context("Failed to find the course url")
}
