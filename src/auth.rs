use eyre::{eyre, Result, WrapErr};

use std::{fs, io::Write};

use crate::{config::Config, state::State, Props, PropsExt};

impl<W: Write> State<W> {
    pub(crate) fn negotiate_otp(&mut self, props: &Props, cfg: &Config) -> Result<Props> {
        match props.get_prop("authentication.type")?.as_str() {
            "ldap" => {
                writeln!(self.out, "Authenticating with ldap")?;
                let user = self.resolve_username(&cfg.username)?;
                let pass = self.resolve_password(&cfg.password)?;

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
                .wrap_err("failed to negotiate one-time password with the server")?;

                fs::write(".submitUser", &submit_user)
                    .wrap_err("failed to write to .submitUser")?;
                Ok(java_properties::read(&*submit_user)?)
            }

            auth => Err(eyre!("unsupported authentication type: {auth}")),
        }
    }
}
