use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

pub struct NjallaClient {
    api_key: String,
    server: String,
}

impl NjallaClient {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let api_key = config.password.as_ref()
            .ok_or("Njalla requires API key (use password)")?
            .clone();
        let server = config.server.clone()
            .unwrap_or_else(|| "https://njal.la".to_string());

        Ok(Self {
            api_key,
            server,
        })
    }
}

impl DnsClient for NjallaClient {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let url = format!("{}/update?h={}&k={}&a={}", 
            self.server, hostname, self.api_key, ip);
        
        log::info!("Updating {} to {}", hostname, ip);
        
        let response = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .send()?;

        // Njalla returns status 200 on success
        if response.status_code == 200 {
            let body = response.as_str()?;
            // Empty response or contains success indicators
            if body.is_empty() || !body.to_lowercase().contains("error") {
                log::info!("Successfully updated {} to {}", hostname, ip);
                return Ok(());
            }
            return Err(format!("Update failed: {}", body).into());
        }

        Err(format!("HTTP error: {}", response.status_code).into())
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.api_key.is_empty() {
            return Err("Njalla API key cannot be empty".into());
        }
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "Njalla"
    }
}
