use anyhow::{anyhow, Context};
use rpassword::prompt_password;

use std::{
    collections::HashMap,
    fs,
    io::{self, Write},
};

pub fn negotiate_otp(props: &HashMap<String, String>) -> anyhow::Result<HashMap<String, String>> {
    match props.get("authentication.type").map(|x| x.as_str()) {
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
                props.get("baseURL").context("baseURL is null in .submit")?
            ))
            .send_form(&[
                ("loginName", &user),
                ("password", &pass),
                (
                    "courseKey",
                    props
                        .get("courseKey")
                        .context("courseKey is null in .submit")?,
                ),
                (
                    "projectNumber",
                    props
                        .get("projectNumber")
                        .context("projectNumber is null in .submit")?,
                ),
            ])
            .and_then(|resp| {
                resp.into_reader()
                    .read_to_end(&mut submit_user)
                    .map_err(Into::into)
            })
            .context("Failed to negotiate one-time password with the server")?;

            fs::write(".submitUser", &submit_user).context("Failed to write to .submitUser")?;
            Ok(java_properties::read(&*submit_user)?)
        }

        Some(auth) => Err(anyhow!("Unsupported authentication type: {auth}")),

        _ => Err(anyhow!("authentication.type is null in .submit")),
    }
}
