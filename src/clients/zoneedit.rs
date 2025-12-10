use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;
use base64::{Engine as _, engine::general_purpose};

/// Zoneedit client - DynDNS2-compatible
pub struct ZoneeditClient {
    username: String,
    password: String,
    server: String,
}

impl ZoneeditClient {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let username = config.login.as_ref()
            .ok_or("username is required for Zoneedit")?
            .clone();
        let password = config.password.as_ref()
            .ok_or("password is required for Zoneedit")?
            .clone();
        
        let server = config.server.as_ref()
            .cloned()
            .unwrap_or_else(|| "https://dynamic.zoneedit.com".to_string());

        Ok(ZoneeditClient {
            username,
            password,
            server,
        })
    }
}

impl DnsClient for ZoneeditClient {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let url = format!(
            "{}/auth/dynamic.html?host={}&dnsto={}",
            self.server, hostname, ip
        );

        log::info!("Updating {} with Zoneedit", hostname);

        let response = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .with_header("Authorization", format!("Basic {}", 
                general_purpose::STANDARD.encode(format!("{}:{}", self.username, self.password))))
            .send()?;

        let body = response.as_str()?;

        if body.contains("<SUCCESS") {
            log::info!("DNS record for {} successfully updated to {}", hostname, ip);
            Ok(())
        } else if body.contains("<ERROR") {
            Err("Zoneedit update failed - check credentials and hostname".into())
        } else {
            Err(format!("Unexpected response: {}", body).into())
        }
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.username.is_empty() {
            return Err("username is required for Zoneedit".into());
        }
        if self.password.is_empty() {
            return Err("password is required for Zoneedit".into());
        }
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "Zoneedit"
    }
}
