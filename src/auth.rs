use anyhow::{anyhow, Context, Result};

use std::fs;

use crate::{
    cli::Opts,
    cred::{resolve_password, resolve_username},
    Props, PropsExt,
};

pub fn negotiate_otp(props: &Props, opts: &Opts) -> Result<Props> {
    match props.get_prop("authentication.type")?.as_str() {
        "ldap" => {
            println!("Authenticating with ldap...");
            let user = resolve_username(opts)?;
            let pass = resolve_password(opts)?;

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
