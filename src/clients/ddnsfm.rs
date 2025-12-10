use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

/// DDNS.FM DNS client
/// Uses DDNS.FM REST API
pub struct DdnsfmClient {
    server: String,
    token: String,
}

impl DdnsfmClient {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let token = config.password.as_ref()
            .ok_or("password (token) is required for DDNS.FM")?
            .clone();
        
        let server = config.server.clone()
            .unwrap_or_else(|| "https://api.ddns.fm".to_string());

        Ok(DdnsfmClient {
            server,
            token,
        })
    }
}

impl DnsClient for DdnsfmClient {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        log::info!("Updating {} with DDNS.FM", hostname);

        // DDNS.FM API endpoint
        let url = format!("{}/update", self.server);

        let response = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .with_param("token", &self.token)
            .with_param("hostname", hostname)
            .with_param("ip", ip.to_string())
            .send()?;

        let status_code = response.status_code;
        let body = response.as_str()?.trim();

        log::debug!("Response status: {}, body: {}", status_code, body);

        if status_code != 200 {
            return Err(format!("HTTP error: {}", status_code).into());
        }

        // Check for success indicators
        if body.contains("success") || body.contains("updated") || body == "OK" {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else if body.contains("error") || body.contains("fail") {
            Err(format!("DDNS.FM error: {}", body).into())
        } else {
            Err(format!("Unexpected response: {}", body).into())
        }
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.token.is_empty() {
            return Err("password (token) is required for DDNS.FM".into());
        }
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "DDNS.FM"
    }
}
