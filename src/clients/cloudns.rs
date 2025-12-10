use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

pub struct CloudnsClient {
    dynurl: String,
}

impl CloudnsClient {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        // ClouDNS uses a unique dynamic URL per host
        let dynurl = config.password.as_ref()
            .or(config.server.as_ref())
            .ok_or("ClouDNS requires dynurl (use password or server config)")?
            .clone();

        Ok(Self {
            dynurl,
        })
    }
}

impl DnsClient for CloudnsClient {
    fn update_record(&self, _hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        // ClouDNS dynurl already contains the hostname, just append IP
        let url = if self.dynurl.contains('?') {
            format!("{}&myip={}", self.dynurl, ip)
        } else {
            format!("{}?myip={}", self.dynurl, ip)
        };
        
        log::info!("Updating ClouDNS record to {}", ip);
        
        let response = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .send()?;

        if response.status_code != 200 {
            return Err(format!("HTTP error: {}", response.status_code).into());
        }

        let body = response.as_str()?;
        
        // ClouDNS typically returns success indicators
        if body.to_lowercase().contains("success") 
            || body.contains("good") 
            || body.contains("updated") {
            log::info!("Successfully updated to {}", ip);
            Ok(())
        } else if body.to_lowercase().contains("error") 
            || body.to_lowercase().contains("fail") {
            Err(format!("Update failed: {}", body).into())
        } else {
            // Assume success if no error indicator
            log::warn!("Unclear response, assuming success: {}", body);
            Ok(())
        }
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.dynurl.is_empty() {
            return Err("ClouDNS dynurl cannot be empty".into());
        }
        if !self.dynurl.starts_with("http://") && !self.dynurl.starts_with("https://") {
            return Err("ClouDNS dynurl must start with http:// or https://".into());
        }
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "ClouDNS"
    }
}
