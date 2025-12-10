use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

/// Selfhost.de DNS client
/// Uses Selfhost.de DynDNS2 protocol
pub struct SelfhostClient {
    server: String,
    username: String,
    password: String,
}

impl SelfhostClient {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let username = config.login.as_ref()
            .ok_or("username is required for Selfhost.de")?
            .clone();
        
        let password = config.password.as_ref()
            .ok_or("password is required for Selfhost.de")?
            .clone();
        
        let server = config.server.clone()
            .unwrap_or_else(|| "https://carol.selfhost.de".to_string());

        Ok(SelfhostClient {
            server,
            username,
            password,
        })
    }
}

impl DnsClient for SelfhostClient {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        log::info!("Updating {} with Selfhost.de", hostname);

        // Selfhost.de DynDNS2 compatible endpoint
        let url = format!("{}/nic/update", self.server);

        let auth = format!("{}:{}", self.username, self.password);
        let encoded_auth = format!("Basic {}", base64::encode(&auth));

        let response = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .with_header("Authorization", &encoded_auth)
            .with_param("hostname", hostname)
            .with_param("myip", ip.to_string())
            .send()?;

        let status_code = response.status_code;
        let body = response.as_str()?.trim();

        log::debug!("Response status: {}, body: {}", status_code, body);

        if status_code != 200 {
            return Err(format!("HTTP error: {}", status_code).into());
        }

        // DynDNS2 protocol response codes
        if body.starts_with("good") || body.starts_with("nochg") {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else if body.starts_with("badauth") {
            Err("Authentication failed".into())
        } else if body.starts_with("notfqdn") {
            Err("Invalid hostname".into())
        } else if body.starts_with("nohost") {
            Err("Hostname not found".into())
        } else if body.starts_with("abuse") {
            Err("Account blocked for abuse".into())
        } else {
            Err(format!("Update failed: {}", body).into())
        }
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.username.is_empty() {
            return Err("username is required for Selfhost.de".into());
        }
        if self.password.is_empty() {
            return Err("password is required for Selfhost.de".into());
        }
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "Selfhost.de"
    }
}

mod base64 {
    use base64::{Engine as _, engine::general_purpose};
    
    pub fn encode(data: &str) -> String {
        general_purpose::STANDARD.encode(data.as_bytes())
    }
}
