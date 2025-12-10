use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

/// Afraid.org DNS client (v2 API)
/// Uses Afraid.org's update API with token
pub struct AfraidClient {
    server: String,
    token: String,
}

impl AfraidClient {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let token = config.api_token.as_ref()
            .ok_or("api_token (update token) is required for Afraid.org")?
            .clone();
        
        let server = config.server.clone()
            .unwrap_or_else(|| "https://freedns.afraid.org".to_string());

        Ok(AfraidClient {
            server,
            token,
        })
    }
}

impl DnsClient for AfraidClient {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        log::info!("Updating {} with Afraid.org", hostname);

        // Afraid.org API endpoint with token
        let url = format!("{}/api/?action=getdyndns&sha={}", self.server, self.token);

        let response = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .with_param("hostname", hostname)
            .with_param("myip", &ip.to_string())
            .send()?;

        let status_code = response.status_code;
        let body = response.as_str()?.trim();

        log::debug!("Response status: {}, body: {}", status_code, body);

        if status_code != 200 {
            return Err(format!("HTTP error: {}", status_code).into());
        }

        // Check for success indicators
        if body.contains("Updated") || body.contains("has not changed") {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else if body.contains("ERROR") {
            Err(format!("Afraid.org error: {}", body).into())
        } else {
            Err(format!("Unexpected response: {}", body).into())
        }
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.token.is_empty() {
            return Err("api_token (update token) is required for Afraid.org".into());
        }
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "Afraid.org"
    }
}
