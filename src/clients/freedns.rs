use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

/// Freedns (afraid.org) client - https://freedns.afraid.org/
pub struct FreednsClient {
    token: String,
    server: String,
}

impl FreednsClient {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let token = config.password.as_ref()
            .or(config.api_token.as_ref())
            .ok_or("token (password or api_token) is required for Freedns")?
            .clone();
        
        let server = config.server.as_ref()
            .cloned()
            .unwrap_or_else(|| "https://freedns.afraid.org/dynamic".to_string());

        Ok(FreednsClient { token, server })
    }
}

impl DnsClient for FreednsClient {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        // Freedns uses a unique token per host
        let url = format!("{}/update.php?{}&address={}", self.server, self.token, ip);

        log::info!("Updating {} with Freedns", hostname);

        let response = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .send()?;

        let body = response.as_str()?;

        if body.contains("Updated") || body.contains("has not changed") {
            log::info!("DNS record for {} successfully updated to {}", hostname, ip);
            Ok(())
        } else if body.contains("ERROR") {
            Err(format!("Freedns error: {}", body).into())
        } else {
            Err(format!("Unexpected response: {}", body).into())
        }
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.token.is_empty() {
            return Err("token is required for Freedns".into());
        }
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "Freedns"
    }
}
