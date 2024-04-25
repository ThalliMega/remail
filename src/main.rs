#![doc = include_str!("../README.md")]

/*
remail
Copyright (C) 2024 Thallium Megalovania

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as published
by the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/
use std::{
    error::Error,
    io::{stdin, Read},
    time::Duration,
};

use clap::Parser;
use lettre::{
    message::{DkimConfig, DkimSigningKey},
    transport::smtp::authentication::Credentials,
    Message, SmtpTransport, Transport,
};

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// email sent from
    #[arg(long, short, env = "REMAIL_FROM")]
    from: String,
    /// email sent to. can be set multiple times.
    #[arg(long, short)]
    to: Vec<String>,
    /// carbon copy. can be set multiple times.
    #[arg(long)]
    cc: Vec<String>,
    /// blind carbon copy. can be set multiple times.
    #[arg(long)]
    bcc: Vec<String>,
    /// email subject
    #[arg(long, short)]
    subject: String,
    /// dkim selector
    #[arg(long, env = "REMAIL_DKIM_SELECTOR")]
    selector: Option<String>,
    /// key fingerprint used in gpg key search
    #[arg(long, env = "REMAIL_KEY_FINGERPRINT")]
    fingerprint: Option<String>,
    /// the smtp server address
    #[arg(long, env = "REMAIL_SMTP_ADDRESS")]
    server: String,
    /// the smtp server port
    #[arg(long, env = "REMAIL_SMTP_PORT", default_value_t = 25)]
    port: u16,
    /// the smtp auth username
    #[arg(long, env = "REMAIL_SMTP_USERNAME")]
    username: Option<String>,
    /// the smtp auth password
    #[arg(long, env = "REMAIL_SMTP_PASSWORD")]
    password: Option<String>,
    /// timeout in milliseconds
    #[arg(long, env = "REMAIL_TIMEOUT", default_value_t = 10_000)]
    timeout: u64,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();
    let mut body = String::new();
    stdin().read_to_string(&mut body).unwrap();

    let mut msg = Message::builder().from(args.from.parse()?);
    for to in args.to {
        msg = msg.to(to.parse()?);
    }
    for cc in args.cc {
        msg = msg.cc(cc.parse()?);
    }
    for bcc in args.bcc {
        msg = msg.bcc(bcc.parse()?);
    }
    let mut msg = msg.subject(args.subject).body(body)?;

    if let (Some(sel), Some(fpr)) = (args.selector, args.fingerprint) {
        let dkim = DkimConfig::default_config(sel, args.from, DkimSigningKey::new_gpg(fpr)?);
        msg.sign(&dkim);
    }
    let mut sender = SmtpTransport::starttls_relay(&args.server)?
        .port(args.port)
        .timeout(Some(Duration::from_millis(args.timeout)));
    if let (Some(name), Some(pwd)) = (args.username, args.password) {
        sender = sender.credentials(Credentials::new(name, pwd));
    }
    let sender = sender.build();

    println!("Sending...");
    sender.send(&msg)?;
    println!("Done.");
    Ok(())
}
