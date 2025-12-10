use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

pub struct DirectnicClient {
    urlv4: Option<String>,
    urlv6: Option<String>,
}

impl DirectnicClient {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        // For Directnic, we use server for urlv4 and password for urlv6
        let urlv4 = config.server.clone();
        let urlv6 = config.password.clone();

        // At least one URL must be provided
        if urlv4.is_none() && urlv6.is_none() {
            return Err("At least one of urlv4 (server) or urlv6 (password) is required for Directnic".into());
        }

        Ok(DirectnicClient {
            urlv4,
            urlv6,
        })
    }
}

impl DnsClient for DirectnicClient {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        log::info!("Updating Directnic record for {} to {}", hostname, ip);

        // Select the appropriate URL based on IP address type
        let url = match ip {
            IpAddr::V4(_) => {
                self.urlv4.as_ref().ok_or("urlv4 not configured for IPv4 address")?
            }
            IpAddr::V6(_) => {
                self.urlv6.as_ref().ok_or("urlv6 not configured for IPv6 address")?
            }
        };

        // Directnic uses a simple GET request to the provided URL
        let response = minreq::get(url)
            .with_header("User-Agent", crate::USER_AGENT)
            .send()?;

        if response.status_code == 200 {
            log::info!("Successfully updated DNS record for {} to {}", hostname, ip);
            Ok(())
        } else {
            let body = response.as_str().unwrap_or("No response body");
            Err(format!(
                "Directnic API error: HTTP {} - {}",
                response.status_code, body
            )
            .into())
        }
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.urlv4.is_none() && self.urlv6.is_none() {
            return Err("At least one of urlv4 or urlv6 must be configured for Directnic".into());
        }
        
        // Validate URLs if provided
        if let Some(url) = &self.urlv4 {
            if !url.starts_with("http://") && !url.starts_with("https://") {
                return Err("urlv4 must start with http:// or https://".into());
            }
        }
        if let Some(url) = &self.urlv6 {
            if !url.starts_with("http://") && !url.starts_with("https://") {
                return Err("urlv6 must start with http:// or https://".into());
            }
        }
        
        Ok(())
    }

    fn provider_name(&self) -> &'static str {
        "Directnic"
    }
}
