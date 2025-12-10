use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;
use base64::{Engine as _, engine::general_purpose};

/// Regfish DNS client
/// Uses DynDNS2 protocol with Regfish's server
pub struct RegfishClient {
    server: String,
    username: String,
    password: String,
}

impl RegfishClient {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let username = config.login.as_ref()
            .ok_or("username is required for Regfish")?
            .clone();
        let password = config.password.as_ref()
            .ok_or("password is required for Regfish")?
            .clone();
        
        let server = config.server.as_ref()
            .cloned()
            .unwrap_or_else(|| "https://dyndns.regfish.de".to_string());

        Ok(RegfishClient {
            server,
            username,
            password,
        })
    }
}

impl DnsClient for RegfishClient {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let url = format!(
            "{}/?fqdn={}&forcehost=1&authtype=secure&token={}",
            self.server, hostname, self.password
        );

        log::info!("Updating {} with Regfish", hostname);

        let response = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .with_header("Authorization", format!("Basic {}", 
                general_purpose::STANDARD.encode(format!("{}:{}", self.username, self.password))))
            .send()?;

        let status_code = response.status_code;
        let body = response.as_str()?.trim();

        log::debug!("Response status: {}, body: {}", status_code, body);

        if status_code != 200 {
            return Err(format!("HTTP error: {}", status_code).into());
        }

        // Parse DynDNS2-style response
        if body.starts_with("good") || body.starts_with("nochg") {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else if body.starts_with("badauth") {
            Err("Authentication failed - check username and password".into())
        } else if body.starts_with("notfqdn") {
            Err("Invalid hostname format".into())
        } else if body.starts_with("nohost") {
            Err("Hostname does not exist".into())
        } else if body.starts_with("abuse") {
            Err("Account blocked for abuse".into())
        } else if body.starts_with("911") {
            Err("Server error - try again later".into())
        } else {
            Err(format!("Unexpected response: {}", body).into())
        }
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.username.is_empty() {
            return Err("username is required for Regfish".into());
        }
        if self.password.is_empty() {
            return Err("password is required for Regfish".into());
        }
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "Regfish"
    }
}
