use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

/// Freemyip DNS client
/// Uses simple token-based GET protocol
pub struct FreemyipClient {
    server: String,
    token: String,
}

impl FreemyipClient {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let token = config.password.as_ref()
            .ok_or("password (token) is required for Freemyip")?
            .clone();
        
        let server = config.server.as_ref()
            .cloned()
            .unwrap_or_else(|| "https://freemyip.com".to_string());

        Ok(FreemyipClient {
            server,
            token,
        })
    }
}

impl DnsClient for FreemyipClient {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let url = format!(
            "{}/update?token={}&domain={}",
            self.server, self.token, hostname
        );

        log::info!("Updating {} with Freemyip", hostname);

        let response = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .send()?;

        let status_code = response.status_code;
        let body = response.as_str()?.trim();

        log::debug!("Response status: {}, body: {}", status_code, body);

        if status_code != 200 {
            return Err(format!("HTTP error: {}", status_code).into());
        }

        // Parse Freemyip response
        if body.contains("SUCCESS") || body.contains("UPDATED") || body == "OK" {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else if body.contains("ERROR") {
            Err(format!("Freemyip error: {}", body).into())
        } else {
            Err(format!("Unexpected response: {}", body).into())
        }
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.token.is_empty() {
            return Err("password (token) is required for Freemyip".into());
        }
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "Freemyip"
    }
}
