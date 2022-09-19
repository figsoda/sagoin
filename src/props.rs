use std::{fs::File, io::Read};

use anyhow::Context;
use java_properties::PropertiesIter;
use multipart::client::lazy::Multipart;

#[derive(Default)]
pub struct Props {
    pub auth: Option<String>,
    pub base_url: Option<String>,
    pub course_key: Option<String>,
    pub project: Option<String>,
    pub url: Option<String>,

    pub class: Option<String>,
    pub cvs: Option<String>,
    pub login: Option<String>,
    pub otp: Option<String>,

    pub parts: Multipart<'static, 'static>,
}

pub fn read_submit() -> anyhow::Result<Props> {
    let mut props = Props::default();

    PropertiesIter::new(File::open(".submit").context("Failed to read .submit")?)
        .read_into(|k, v| {
            match k.as_ref() {
                "authentication.type" => props.auth = Some(v.clone()),
                "baseURL" => props.base_url = Some(v.clone()),
                "courseKey" => props.course_key = Some(v.clone()),
                "projectNumber" => props.project = Some(v.clone()),
                "submitURL" => {
                    props.url = Some(v);
                    return;
                }
                _ => {}
            };
            props.parts.add_text(k, v);
        })
        .context("Failed to parse .submit")?;

    Ok(props)
}

pub fn read_submit_user(props: &mut Props, submit_user: impl Read) {
    if let Err(e) = PropertiesIter::new(submit_user).read_into(|k, v| {
        match k.as_ref() {
            "classAccount" => props.class = Some(v.clone()),
            "cvsAccount" => props.cvs = Some(v.clone()),
            "loginName" => props.login = Some(v.clone()),
            "oneTimePassword" => props.otp = Some(v.clone()),
            _ => {}
        }
        props.parts.add_text(k, v);
    }) {
        eprintln!("Warning: error when parsing .submitUser: {e}");
    }
}
