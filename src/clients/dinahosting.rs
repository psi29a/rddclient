use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

/// Dinahosting DNS client
/// Uses Dinahosting's REST API with basic authentication
pub struct DinahostingClient {
    server: String,
    username: String,
    password: String,
}

impl DinahostingClient {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let username = config.login.as_ref()
            .ok_or("username is required for Dinahosting")?
            .clone();
        let password = config.password.as_ref()
            .ok_or("password is required for Dinahosting")?
            .clone();
        
        let server = config.server.as_ref()
            .cloned()
            .unwrap_or_else(|| "https://dinahosting.com".to_string());

        Ok(DinahostingClient {
            server,
            username,
            password,
        })
    }

    fn get_domain_from_hostname(&self, hostname: &str) -> String {
        // Extract domain from hostname (e.g., "ddns.example.com" -> "example.com")
        hostname.split('.').skip(1).collect::<Vec<_>>().join(".")
    }
}

impl DnsClient for DinahostingClient {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let domain = self.get_domain_from_hostname(hostname);
        let record_type = match ip {
            IpAddr::V4(_) => "A",
            IpAddr::V6(_) => "AAAA",
        };

        let url = format!(
            "{}/special/api.php?command=Domain_Zone_UpdateDynDNS&domain={}&zone={}&type={}&ip={}",
            self.server, domain, hostname, record_type, ip
        );

        log::info!("Updating {} with Dinahosting", hostname);

        // Use HTTP Basic Auth header instead of URL parameters for security
        let auth = format!("{}:{}", self.username, self.password);
        use base64::{Engine as _, engine::general_purpose};
        let encoded_auth = general_purpose::STANDARD.encode(auth.as_bytes());

        let response = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .with_header("Authorization", &format!("Basic {}", encoded_auth))
            .send()?;

        let status_code = response.status_code;
        let body = response.as_str()?.trim();

        log::debug!("Response status: {}, body: {}", status_code, body);

        if status_code != 200 {
            return Err(format!("HTTP error: {}", status_code).into());
        }

        // Parse response
        if body.contains("responseStatus=ok") || body.contains("success") {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else if body.contains("responseStatus=error") {
            if body.contains("authentication") {
                Err("Authentication failed - check username and password".into())
            } else {
                Err(format!("Dinahosting error: {}", body).into())
            }
        } else {
            Err(format!("Unexpected response: {}", body).into())
        }
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.username.is_empty() {
            return Err("username is required for Dinahosting".into());
        }
        if self.password.is_empty() {
            return Err("password is required for Dinahosting".into());
        }
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "Dinahosting"
    }
}
