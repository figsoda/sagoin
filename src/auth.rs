use anyhow::{anyhow, Context};
use rpassword::prompt_password;

use std::{
    fs,
    io::{self, Write},
};

use crate::props::{read_submit, read_submit_user, Props};

pub fn negotiate_otp(props: Props) -> anyhow::Result<Props> {
    match props.auth.as_deref() {
        Some("ldap") => {
            print!("Authenticating with ldap...\nUsername: ");
            io::stdout()
                .flush()
                .context("Failed to prompt for username")?;

            let mut user = String::new();
            io::stdin()
                .read_line(&mut user)
                .context("Failed to prompt for username")?;

            let pass = prompt_password("Password: ").context("Failed to prompt for password")?;

            let mut submit_user = Vec::new();
            ureq::post(&format!(
                "{}/eclipse/NegotiateOneTimePassword",
                props
                    .base_url
                    .as_ref()
                    .context("baseURL is null in .submit")?
            ))
            .send_form(&[
                ("loginName", &user),
                ("password", &pass),
                (
                    "courseKey",
                    props
                        .course_key
                        .as_ref()
                        .context("courseKey is null in .submit")?,
                ),
                (
                    "projectNumber",
                    props
                        .project
                        .as_ref()
                        .context("projectNumber is null in .submit")?,
                ),
            ])
            .and_then(|resp| {
                resp.into_reader()
                    .read_to_end(&mut submit_user)
                    .map_err(Into::into)
            })
            .context("Failed to negotiate one-time password with the server")?;

            let mut submit = read_submit()?;
            read_submit_user(&mut submit, &*submit_user);
            fs::write(".submitUser", submit_user).context("Failed to write to .submitUser")?;

            Ok(submit)
        }

        Some(auth) => Err(anyhow!("Unsupported authentication type: {auth}")),

        _ => Err(anyhow!("authentication.type is null in .submit")),
    }
}
