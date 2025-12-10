use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

/// Key-Systems (RRPproxy) DNS client
/// Uses Key-Systems dynamic DNS API
pub struct KeysystemsClient {
    server: String,
    token: String,
}

impl KeysystemsClient {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let token = config.password.as_ref()
            .ok_or("password (token) is required for Key-Systems")?
            .clone();
        
        let server = config.server.clone()
            .unwrap_or_else(|| "https://dynamicdns.key-systems.net".to_string());

        Ok(KeysystemsClient {
            server,
            token,
        })
    }
}

impl DnsClient for KeysystemsClient {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        log::info!("Updating {} with Key-Systems", hostname);

        // Key-Systems dynamic DNS endpoint
        let url = format!("{}/nic/update", self.server);

        let response = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .with_param("token", &self.token)
            .with_param("hostname", hostname)
            .with_param("myip", &ip.to_string())
            .send()?;

        let status_code = response.status_code;
        let body = response.as_str()?.trim();

        log::debug!("Response status: {}, body: {}", status_code, body);

        if status_code != 200 {
            return Err(format!("HTTP error: {}", status_code).into());
        }

        // Key-Systems uses DynDNS-like response codes
        if body.starts_with("good") || body.starts_with("nochg") {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else if body.starts_with("badauth") {
            Err("Authentication failed - invalid token".into())
        } else if body.starts_with("notfqdn") {
            Err("Invalid hostname format".into())
        } else if body.starts_with("nohost") {
            Err("Hostname not found in your account".into())
        } else if body.starts_with("abuse") {
            Err("Account blocked for abuse".into())
        } else if body.starts_with("badagent") {
            Err("User agent blocked".into())
        } else {
            Err(format!("Update failed: {}", body).into())
        }
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.token.is_empty() {
            return Err("password (token) is required for Key-Systems".into());
        }
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "Key-Systems"
    }
}
