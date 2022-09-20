#![forbid(unsafe_code)]

pub mod auth;
pub mod cli;

use anyhow::{anyhow, Context, Result};
use icalendar::parser::{read_calendar_simple, unfold};
use multipart::client::lazy::Multipart;

use std::{collections::HashMap, io::Cursor};

use crate::auth::negotiate_otp;

pub type Props = HashMap<String, String>;

trait PropsExt {
    fn get_prop(&self, key: &'static str) -> Result<&String>;
}

impl PropsExt for Props {
    fn get_prop(&self, key: &'static str) -> Result<&String> {
        self.get(key).context(format!("{key} is null in .submit"))
    }
}

pub fn submit(user_props: Props, props: &Props, zip: Cursor<Vec<u8>>, reauth: bool) -> Result<()> {
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

    match ureq::post(props.get_prop("submitURL")?)
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
        Err(e) => Err(e).context("Failed to send request to the submit server"),
    }
}

pub fn get_course_url(props: &Props) -> Result<String> {
    let proj = format!(
        "{} project {}: ",
        props.get_prop("courseName")?,
        props.get_prop("projectNumber")?,
    );

    read_calendar_simple(&unfold(
        &ureq::get(&format!(
            "{}/feed/CourseCalendar?courseKey={}",
            props.get_prop("baseURL")?,
            props.get_prop("courseKey")?,
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