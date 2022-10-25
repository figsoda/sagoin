use eyre::{eyre, Result, WrapErr};
use icalendar::parser::{read_calendar_simple, unfold};

use crate::{Props, PropsExt};

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
