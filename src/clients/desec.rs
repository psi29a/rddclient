use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

/// deSEC DNS client
/// Uses deSEC REST API
pub struct DesecClient {
    server: String,
    token: String,
    domain: String,
}

impl DesecClient {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let token = config.api_token.as_ref()
            .ok_or("api_token is required for deSEC")?
            .clone();
        
        let domain = config.zone_id.as_ref()
            .ok_or("zone_id (domain) is required for deSEC")?
            .clone();
        
        let server = config.server.clone()
            .unwrap_or_else(|| "https://update.dedyn.io".to_string());

        Ok(DesecClient {
            server,
            token,
            domain,
        })
    }
}

impl DnsClient for DesecClient {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        log::info!("Updating {} with deSEC", hostname);

        // Extract subdomain from hostname
        let subdomain = if hostname.ends_with(&self.domain) {
            hostname.strip_suffix(&format!(".{}", self.domain))
                .unwrap_or("")
        } else {
            hostname
        };

        // deSEC update endpoint (DynDNS2 compatible)
        let url = format!("{}/update", self.server);

        let auth = format!("{}:{}", self.domain, self.token);
        let encoded_auth = format!("Basic {}", base64::encode(&auth));

        let response = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .with_header("Authorization", &encoded_auth)
            .with_param("hostname", if subdomain.is_empty() { &self.domain } else { hostname })
            .with_param("myip", &ip.to_string())
            .send()?;

        let status_code = response.status_code;
        let body = response.as_str()?.trim();

        log::debug!("Response status: {}, body: {}", status_code, body);

        if status_code != 200 {
            return Err(format!("HTTP error: {}", status_code).into());
        }

        // deSEC returns status codes similar to DynDNS2
        if body.starts_with("good") || body.starts_with("nochg") {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else if body.starts_with("badauth") {
            Err("Authentication failed".into())
        } else if body.starts_with("notfqdn") {
            Err("Invalid hostname".into())
        } else {
            Err(format!("Update failed: {}", body).into())
        }
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.token.is_empty() {
            return Err("api_token is required for deSEC".into());
        }
        if self.domain.is_empty() {
            return Err("zone_id (domain) is required for deSEC".into());
        }
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "deSEC"
    }
}

mod base64 {
    use base64::{Engine as _, engine::general_purpose};
    
    pub fn encode(data: &str) -> String {
        general_purpose::STANDARD.encode(data.as_bytes())
    }
}
