use crate::clients::DnsClient;
use crate::config::Config;
use base64::{Engine as _, engine::general_purpose};
use std::error::Error;
use std::net::IpAddr;

pub struct DomeneshopClient {
    username: String,
    password: String,
    server: String,
}

impl DomeneshopClient {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let username = config.login.as_ref()
            .ok_or("Domeneshop requires username (API token)")?
            .clone();
        let password = config.password.as_ref()
            .ok_or("Domeneshop requires password (API secret)")?
            .clone();
        let server = config.server.clone()
            .unwrap_or_else(|| "https://api.domeneshop.no".to_string());

        Ok(Self {
            username,
            password,
            server,
        })
    }
}

impl DnsClient for DomeneshopClient {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let auth = general_purpose::STANDARD.encode(format!("{}:{}", self.username, self.password));
        
        let url = format!("{}/v0/dyndns/update?hostname={}&myip={}", 
            self.server, hostname, ip);
        
        log::info!("Updating {} to {}", hostname, ip);
        
        let response = minreq::get(&url)
            .with_header("Authorization", format!("Basic {}", auth))
            .with_header("User-Agent", crate::USER_AGENT)
            .send()?;

        if response.status_code != 200 && response.status_code != 204 {
            return Err(format!("HTTP error: {}", response.status_code).into());
        }

        let body = response.as_str()?;
        
        // Check for success
        if body.is_empty() || body.contains("good") || body.contains("nochg") {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else if body.contains("badauth") {
            Err("Bad authorization (invalid credentials)".into())
        } else if body.contains("nohost") {
            Err("Hostname does not exist".into())
        } else {
            Err(format!("Update failed: {}", body).into())
        }
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.username.is_empty() {
            return Err("Domeneshop username cannot be empty".into());
        }
        if self.password.is_empty() {
            return Err("Domeneshop password cannot be empty".into());
        }
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "Domeneshop"
    }
}
