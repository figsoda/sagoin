use std::io::{stdout, Write};

use eyre::{eyre, Report, Result, WrapErr};
use icalendar::parser::{read_calendar_simple, unfold, Component};
use time::{format_description, macros::format_description, PrimitiveDateTime};

use crate::{Props, PropsExt};

#[derive(Default)]
pub struct CourseInfo {
    summary: String,
    due: Option<String>,
    description: Option<String>,
    url: Option<String>,
}

pub fn print_course_info(props: &Props, fmt: String) -> Result<()> {
    let info = get_course_info(props)?;
    let mut out = stdout().lock();

    writeln!(out, "{}", info.summary)?;
    if let Some(due) = info.due {
        write!(out, "Due: ")?;

        PrimitiveDateTime::parse(
            &due,
            format_description!("[year][month][day]T[hour][minute][second]Z"),
        )
        .wrap_err("failed to parse time")?
        .format_into(
            &mut out,
            &format_description::parse(&fmt).wrap_err("failed to parse time format")?,
        )
        .wrap_err("failed to format time")?;

        writeln!(out)?;
    }
    if let Some(description) = info.description {
        // https://github.com/hoodie/icalendar-rs/issues/53
        writeln!(out, "{}", description.replace('\\', ""))?;
    }
    if let Some(url) = info.url {
        writeln!(out, "{url}")?;
    }

    Ok(())
}

fn get_course_info(props: &Props) -> Result<CourseInfo> {
    get_course_props(
        props,
        |component, prefix| {
            let mut summary = None;
            let mut due = None;
            let mut description = None;
            let mut url = None;

            for prop in component.properties.iter() {
                match prop.name.as_str() {
                    "SUMMARY" => {
                        let s = prop.val.to_string();
                        if s.starts_with(prefix) {
                            summary = Some(s);
                        } else {
                            return None;
                        }
                    }

                    "DTSTART" => due = Some(prop.val.to_string()),

                    "DESCRIPTION" => description = Some(prop.val.to_string()),

                    "URL" => url = Some(prop.val.to_string()),

                    _ => {}
                }
            }

            summary.map(|summary| CourseInfo {
                summary,
                due,
                description,
                url,
            })
        },
        || eyre!("failed to find information for the course"),
    )
}

pub fn get_course_url(props: &Props) -> Result<String> {
    get_course_props(
        props,
        |component, prefix| {
            let mut url = None;
            let mut found = false;

            for prop in component.properties.iter() {
                match prop.name.as_str() {
                    "SUMMARY" => {
                        if prop.val.as_str().starts_with(prefix) {
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
        },
        || eyre!("failed to find the course url"),
    )
}

fn get_course_props<A>(
    props: &Props,
    f: fn(&Component, &str) -> Option<A>,
    e: fn() -> Report,
) -> Result<A> {
    let prefix = format!(
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
        root.components
            .iter()
            .find_map(|component| f(component, &prefix))
    })
    .ok_or_else(e)
}
