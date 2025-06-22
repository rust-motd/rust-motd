use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use openssl::asn1::Asn1Time;
use openssl::x509::X509;
use serde::Deserialize;
use std::fs::File;
use std::io::{BufReader, Read};
use termion::{color, style};
use thiserror::Error;

use crate::component::Component;
use crate::config::global_config::GlobalConfig;
use crate::constants::INDENT_WIDTH;
use crate::default_prepare;

const SECS_PER_DAY: i64 = 24 * 60 * 60;

#[derive(knuffel::DecodeScalar, Debug, Deserialize, Default)]
enum SortMethod {
    #[serde(alias = "alphabetical")] // Alias used to match lowercase spelling as well
    Alphabetical,
    #[serde(alias = "expiration")] // Alias used to match lowercase spelling as well
    Expiration,
    #[default]
    #[serde(alias = "manual")] // Alias used to match lowercase spelling as well
    Manual,
}

#[derive(knuffel::Decode, Debug, Deserialize)]
pub struct Cert {
    #[knuffel(property)]
    pub name: String,
    #[knuffel(property)]
    pub path: String,
}

#[derive(knuffel::Decode, Debug, Deserialize)]
pub struct SSLCerts {
    #[serde(default)]
    #[knuffel(property, default)]
    sort_method: SortMethod,
    #[knuffel(children(name = "cert"))]
    #[serde(deserialize_with = "crate::config::toml_config::deserialize_certs")]
    certs: Vec<Cert>,
}

#[async_trait]
impl Component for SSLCerts {
    async fn print(self: Box<Self>, global_config: &GlobalConfig, _width: Option<usize>) {
        self.print_or_error(global_config)
            .unwrap_or_else(|err| println!("SSL Certificate error: {err}"));
        println!();
    }
    default_prepare!();
}

#[derive(Error, Debug)]
pub enum SSLCertsError {
    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    ErrorStack(#[from] openssl::error::ErrorStack),
}

struct CertInfo {
    name: String,
    status: String,
    expiration: DateTime<Utc>,
}

impl SSLCerts {
    pub fn print_or_error(self, global_config: &GlobalConfig) -> Result<(), SSLCertsError> {
        let mut cert_infos: Vec<CertInfo> = Vec::new();

        println!("SSL Certificates:");
        for Cert { name, path } in self.certs {
            let cert = File::open(&path)?;
            let cert = BufReader::new(cert);
            let cert: Vec<u8> = cert.bytes().collect::<Result<_, _>>()?;
            let cert = X509::from_pem(&cert)?;

            let expiration = Asn1Time::from_unix(0)?.diff(cert.not_after())?;
            let seconds = (expiration.days as i64) * SECS_PER_DAY + (expiration.secs as i64);
            let expiration = DateTime::from_timestamp(seconds, 0).unwrap();

            let now = Utc::now();
            let status = if expiration < now {
                format!("{}expired on{}", color::Fg(color::Red), style::Reset)
            } else if expiration < now + Duration::days(30) {
                format!("{}expiring on{}", color::Fg(color::Yellow), style::Reset)
            } else {
                format!("{}valid until{}", color::Fg(color::Green), style::Reset)
            };
            cert_infos.push(CertInfo {
                name,
                status,
                expiration,
            });
        }

        match self.sort_method {
            SortMethod::Alphabetical => {
                cert_infos.sort_by(|a, b| a.name.cmp(&b.name));
            }
            SortMethod::Expiration => {
                cert_infos.sort_by(|a, b| a.expiration.cmp(&b.expiration));
            }
            SortMethod::Manual => {}
        }

        for cert_info in cert_infos.into_iter() {
            println!(
                "{}{} {} {}",
                " ".repeat(INDENT_WIDTH),
                cert_info.name,
                cert_info.status,
                cert_info.expiration.format(&global_config.time_format)
            );
        }

        Ok(())
    }
}
