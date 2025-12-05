use anyhow::Result;
use lettre::{
    Message, SmtpTransport, Transport,
    message::{Mailbox, header::ContentType},
    transport::smtp::authentication::Credentials,
};
use std::fmt::Write;

use crate::hydra::HydraBuild;

pub struct Mailer {
    from: Mailbox,
    to: Mailbox,
    transport: SmtpTransport,
}

impl Mailer {
    pub fn new(
        host: &str,
        from: String,
        to: String,
        username: String,
        password: String,
    ) -> Result<Self> {
        let transport = SmtpTransport::relay(host)?
            .credentials(Credentials::new(username, password))
            .build();
        Ok(Self {
            from: from.parse()?,
            to: to.parse()?,
            transport,
        })
    }

    pub fn send_report(&self, builds: &[&HydraBuild]) -> Result<()> {
        if builds.is_empty() {
            return Ok(());
        }

        let email = Message::builder()
            .from(self.from.clone())
            .to(self.to.clone())
            .subject("Packages failing to build in Nixpkgs")
            .header(ContentType::TEXT_PLAIN)
            .body(Self::build_email_content(builds))?;

        self.transport.send(&email)?;

        Ok(())
    }

    fn build_email_content(builds: &[&HydraBuild]) -> String {
        let mut res = String::new();

        for build in builds {
            writeln!(
                &mut res,
                "- {} - https://hydra.nixos.org/build/{} - {}",
                build.get_full_name(),
                build.id,
                build.build_status_to_str()
            )
            .unwrap();
        }

        res
    }
}
