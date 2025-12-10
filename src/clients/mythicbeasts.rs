use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;
use base64::{Engine as _, engine::general_purpose};

pub struct MythicbeastsClient {
    username: String,
    password: String,
    server: String,
}

impl MythicbeastsClient {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let username = config.login.as_ref()
            .ok_or("username is required for Mythic Beasts")?;
        let password = config.password.as_ref()
            .ok_or("password is required for Mythic Beasts")?;
        let server = config.server.as_deref()
            .unwrap_or("api.mythic-beasts.com");

        Ok(MythicbeastsClient {
            username: username.to_string(),
            password: password.to_string(),
            server: server.to_string(),
        })
    }
}

impl DnsClient for MythicbeastsClient {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        log::info!("Updating Mythic Beasts record for {} to {}", hostname, ip);

        // Mythic Beasts uses separate IPv4 and IPv6 endpoints
        let (ipv, subdomain) = match ip {
            IpAddr::V4(_) => ("4", "ipv4"),
            IpAddr::V6(_) => ("6", "ipv6"),
        };

        let url = format!(
            "https://{}.{}/dns/v2/dynamic/{}",
            subdomain, self.server, hostname
        );

        let auth = general_purpose::STANDARD.encode(format!("{}:{}", self.username, self.password));
        let response = minreq::post(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .with_header("Authorization", format!("Basic {}", auth))
            .send()?;

        if response.status_code == 200 {
            log::info!("Successfully updated IPv{} record for {} to {}", ipv, hostname, ip);
            Ok(())
        } else {
            let body = response.as_str().unwrap_or("No response body");
            Err(format!(
                "Mythic Beasts API error: HTTP {} - {}",
                response.status_code, body
            )
            .into())
        }
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.username.is_empty() {
            return Err("Mythic Beasts username cannot be empty".into());
        }
        if self.password.is_empty() {
            return Err("Mythic Beasts password cannot be empty".into());
        }
        Ok(())
    }

    fn provider_name(&self) -> &'static str {
        "Mythic Beasts"
    }
}
