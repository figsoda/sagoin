use anyhow::{anyhow, Context, Result};
use rpassword::prompt_password;

use std::{
    fs,
    io::{self, Write},
};

use crate::{Props, PropsExt};

pub fn negotiate_otp(props: &Props) -> Result<Props> {
    match props.get_prop("authentication.type")?.as_str() {
        "ldap" => {
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
                props.get_prop("baseURL")?
            ))
            .send_form(&[
                ("loginName", &user),
                ("password", &pass),
                ("courseKey", props.get_prop("courseKey")?),
                ("projectNumber", props.get_prop("projectNumber")?),
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

        auth => Err(anyhow!("Unsupported authentication type: {auth}")),
    }
}
