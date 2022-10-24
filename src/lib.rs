#![forbid(unsafe_code)]

mod auth;
mod cli;
mod cmd;
pub mod config;
mod cred;
pub mod state;

use eyre::{eyre, Result, WrapErr};
use icalendar::parser::{read_calendar_simple, unfold};
use multipart::client::lazy::Multipart;

use std::{collections::HashMap, io::Write};

use crate::{config::Config, state::State};

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

impl<W: Write> State<W> {
    pub fn submit(
        &mut self,
        user_props: Props,
        props: &Props,
        cfg: &Config,
        zip: &[u8],
    ) -> Result<()> {
        self.run_hook(&cfg.pre_submit_hook, "pre-submit")?;
        self.submit_project(user_props, props, cfg, zip, true)?;
        self.run_hook(&cfg.post_submit_hook, "post-submit")?;

        Ok(())
    }

    fn submit_project(
        &mut self,
        user_props: Props,
        props: &Props,
        opts: &Config,
        zip: &[u8],
        reauth: bool,
    ) -> Result<()> {
        if reauth
            && (!user_props.contains_key("cvsAccount") && !user_props.contains_key("classAccount")
                || !user_props.contains_key("oneTimePassword"))
        {
            let user_props = self.negotiate_otp(props, opts)?;
            return self.submit_project(user_props, props, opts, zip, false);
        }

        let mut parts = Multipart::new();

        for (k, v) in user_props.iter().chain(props) {
            parts.add_text(k.clone(), v);
        }

        let parts = parts
            .add_text("submitClientTool", "sagoin")
            .add_text("submitClientVersion", env!("CARGO_PKG_VERSION"))
            .add_stream(
                "submittedFiles",
                zip,
                Some("submit.zip"),
                Some(
                    "application/zip"
                        .parse()
                        .wrap_err("failed to parse application/zip as a mime type")?,
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
                    write!(self.out, "{success}")?;
                } else {
                    writeln!(self.out, "Successful submission received")?;
                }

                Ok(())
            }

            Err(ureq::Error::Status(500, resp)) if reauth => {
                warn!(self, "Status code 500");
                if let Ok(err) = resp.into_string() {
                    warn!(self, "{err}");
                }
                let user_props = self.negotiate_otp(props, opts)?;
                self.submit_project(user_props, props, opts, zip, false)
            }

            Err(ureq::Error::Status(code, resp)) => Err(if let Ok(err) = resp.into_string() {
                eyre!("{}", err.trim_end())
                    .wrap_err(format!("status code {code}"))
                    .wrap_err("failed to submit project")
            } else {
                eyre!("status code {code}").wrap_err("failed to submit project")
            }),

            Err(e) => Err(e).wrap_err("failed to send request to the submit server"),
        }
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
        .wrap_err("failed to download the course calendar")?
        .into_string()
        .wrap_err("failed to parse the course calendar")?,
    ))
    .map_err(|e| eyre!("{e}").wrap_err("failed to parse the course calendar"))?
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
    .ok_or_else(|| eyre!("failed to find the course url"))
}
