use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

/// Woima.fi DNS client
/// Uses Woima.fi DynDNS2 protocol
pub struct WoimaClient {
    server: String,
    username: String,
    password: String,
}

impl WoimaClient {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let username = config.login.as_ref()
            .ok_or("username is required for Woima.fi")?
            .clone();
        
        let password = config.password.as_ref()
            .ok_or("password is required for Woima.fi")?
            .clone();
        
        let server = config.server.clone()
            .unwrap_or_else(|| "https://www.woima.fi".to_string());

        Ok(WoimaClient {
            server,
            username,
            password,
        })
    }
}

impl DnsClient for WoimaClient {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        log::info!("Updating {} with Woima.fi", hostname);

        // Woima.fi DynDNS2 compatible endpoint
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
            return Err("username is required for Woima.fi".into());
        }
        if self.password.is_empty() {
            return Err("password is required for Woima.fi".into());
        }
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "Woima.fi"
    }
}

mod base64 {
    use base64::{Engine as _, engine::general_purpose};
    
    pub fn encode(data: &str) -> String {
        general_purpose::STANDARD.encode(data.as_bytes())
    }
}
