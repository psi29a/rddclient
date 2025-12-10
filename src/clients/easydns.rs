use crate::clients::DnsClient;
use crate::config::Config;
use base64::{Engine as _, engine::general_purpose};
use std::error::Error;
use std::net::IpAddr;

pub struct EasydnsClient {
    username: String,
    password: String,
    server: String,
}

impl EasydnsClient {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let username = config.login.as_ref()
            .ok_or("EasyDNS requires username")?
            .clone();
        let password = config.password.as_ref()
            .ok_or("EasyDNS requires password")?
            .clone();
        let server = config.server.clone()
            .unwrap_or_else(|| "https://api.easydns.com".to_string());

        Ok(Self {
            username,
            password,
            server,
        })
    }
}

impl DnsClient for EasydnsClient {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let auth = general_purpose::STANDARD.encode(format!("{}:{}", self.username, self.password));
        
        let url = format!("{}/dyn/generic.php?hostname={}&myip={}", 
            self.server, hostname, ip);
        
        log::info!("Updating {} to {} (note: EasyDNS requires 10min between updates)", 
            hostname, ip);
        
        let response = minreq::get(&url)
            .with_header("Authorization", format!("Basic {}", auth))
            .with_header("User-Agent", crate::USER_AGENT)
            .send()?;

        if response.status_code != 200 {
            return Err(format!("HTTP error: {}", response.status_code).into());
        }

        let body = response.as_str()?;
        let body_lower = body.to_lowercase();
        
        // EasyDNS error codes
        if body_lower.contains("noaccess") || body_lower.contains("no_auth") {
            Err("Authentication failed (wrong username/password or host/domain)".into())
        } else if body_lower.contains("nohost") {
            Err("Hostname does not exist".into())
        } else if body_lower.contains("error") {
            Err(format!("Update failed: {}", body).into())
        } else if body.contains("OK") || body.contains("good") || body.contains("nochg") {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else {
            // Assume success if no error
            log::info!("Updated {} to {}", hostname, ip);
            Ok(())
        }
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.username.is_empty() {
            return Err("EasyDNS username cannot be empty".into());
        }
        if self.password.is_empty() {
            return Err("EasyDNS password cannot be empty".into());
        }
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "EasyDNS"
    }
}
