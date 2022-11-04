use eyre::{eyre, Result, WrapErr};
use multipart::client::lazy::Multipart;

use std::io::Write;

use crate::{config::Config, state::State, warn, Props, PropsExt};

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
        cfg: &Config,
        zip: &[u8],
        reauth: bool,
    ) -> Result<()> {
        if reauth
            && (!user_props.contains_key("cvsAccount") && !user_props.contains_key("classAccount")
                || !user_props.contains_key("oneTimePassword"))
        {
            let user_props = self.negotiate_otp(props, cfg)?;
            return self.submit_project(user_props, props, cfg, zip, false);
        }

        let mut parts = Multipart::new();

        for (k, v) in user_props.iter().chain(props) {
            parts.add_text(k.clone(), v);
        }

        let parts = parts
            .add_text("submitClientTool", &cfg.client_name)
            .add_text("submitClientVersion", &cfg.client_version)
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
                let user_props = self.negotiate_otp(props, cfg)?;
                self.submit_project(user_props, props, cfg, zip, false)
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
