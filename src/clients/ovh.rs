use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;
use serde_json::json;

/// OVH client - https://api.ovh.com/
/// Note: OVH requires application key/secret and consumer key
pub struct OvhClient {
    application_key: String,
    application_secret: String,
    consumer_key: String,
    server: String,
}

impl OvhClient {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        // username = application_key, password = application_secret, api_token = consumer_key
        let application_key = config.login.as_ref()
            .ok_or("username (application key) is required for OVH")?
            .clone();
        let application_secret = config.password.as_ref()
            .ok_or("password (application secret) is required for OVH")?
            .clone();
        let consumer_key = config.password.as_ref()
            .ok_or("api_token (consumer key) is required for OVH")?
            .clone();
        
        let server = config.server.as_ref()
            .cloned()
            .unwrap_or_else(|| "https://eu.api.ovh.com/1.0".to_string());

        Ok(OvhClient {
            application_key,
            application_secret,
            consumer_key,
            server,
        })
    }

    fn parse_hostname(&self, hostname: &str) -> (String, String) {
        let parts: Vec<&str> = hostname.rsplitn(3, '.').collect();
        if parts.len() >= 3 {
            let domain = format!("{}.{}", parts[1], parts[0]);
            let subdomain = parts[2];
            (subdomain.to_string(), domain)
        } else {
            ("".to_string(), hostname.to_string())
        }
    }
}

impl DnsClient for OvhClient {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let (subdomain, domain) = self.parse_hostname(hostname);
        
        // Note: Full OVH implementation requires request signing
        // This is a simplified version - production use requires proper signing
        let url = format!("{}/domain/zone/{}/record", self.server, domain);

        log::info!("Updating {} with OVH (simplified API)", hostname);
        log::warn!("OVH implementation requires proper request signing for production use");

        let body = json!({
            "fieldType": "A",
            "subDomain": subdomain,
            "target": ip.to_string()
        });

        let response = minreq::post(&url)
            .with_header("X-Ovh-Application", &self.application_key)
            .with_header("X-Ovh-Consumer", &self.consumer_key)
            .with_header("Content-Type", "application/json")
            .with_json(&body)?
            .send()?;

        let status_code = response.status_code;

        if status_code == 200 || status_code == 201 {
            log::info!("DNS record for {} successfully updated to {}", hostname, ip);
            Ok(())
        } else {
            let body = response.as_str().unwrap_or("unknown error");
            Err(format!("OVH API error ({}): {}", status_code, body).into())
        }
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.application_key.is_empty() {
            return Err("application key (username) is required for OVH".into());
        }
        if self.application_secret.is_empty() {
            return Err("application secret (password) is required for OVH".into());
        }
        if self.consumer_key.is_empty() {
            return Err("consumer key (api_token) is required for OVH".into());
        }
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "OVH"
    }
}
