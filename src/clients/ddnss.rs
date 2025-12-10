use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

/// DDNSS.de DNS client
/// Uses simple token-based GET protocol
pub struct DdnssClient {
    server: String,
    token: String,
}

impl DdnssClient {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let token = config.password.as_ref()
            .ok_or("password (token) is required for DDNSS")?
            .clone();
        
        let server = config.server.as_ref()
            .cloned()
            .unwrap_or_else(|| "https://www.ddnss.de".to_string());

        Ok(DdnssClient {
            server,
            token,
        })
    }
}

impl DdnssClient {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let url = format!(
            "{}/upd.php?key={}&host={}&ip={}",
            self.server, self.token, hostname, ip
        );

        log::info!("Updating {} with DDNSS", hostname);

        let response = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .send()?;

        let status_code = response.status_code;
        let body = response.as_str()?.trim();

        log::debug!("Response status: {}, body: {}", status_code, body);

        if status_code != 200 {
            return Err(format!("HTTP error: {}", status_code).into());
        }

        // Parse DDNSS response
        if body.contains("Updated") || body.contains("good") {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else if body.contains("badauth") || body.contains("Authentication") {
            Err("Authentication failed - check token".into())
        } else if body.contains("nohost") {
            Err("Hostname does not exist".into())
        } else {
            Err(format!("Unexpected response: {}", body).into())
        }
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.token.is_empty() {
            return Err("password (token) is required for DDNSS".into());
        }
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "DDNSS"
    }
}

impl DnsClient for DdnssClient {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        DdnssClient::update_record(self, hostname, ip)
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        DdnssClient::validate_config(self)
    }

    fn provider_name(&self) -> &str {
        DdnssClient::provider_name(self)
    }
}
