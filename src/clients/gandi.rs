use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;
use serde_json::json;

/// Gandi client - https://api.gandi.net/docs/livedns/
pub struct GandiClient {
    api_key: String,
    server: String,
}

impl GandiClient {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let api_key = config.api_token.as_ref()
            .or(config.password.as_ref())
            .ok_or("api_token or password is required for Gandi")?
            .clone();
        
        let server = config.server.as_ref()
            .cloned()
            .unwrap_or_else(|| "https://api.gandi.net".to_string());

        Ok(GandiClient { api_key, server })
    }

    fn parse_hostname(&self, hostname: &str) -> (String, String) {
        let parts: Vec<&str> = hostname.rsplitn(3, '.').collect();
        if parts.len() >= 3 {
            let domain = format!("{}.{}", parts[1], parts[0]);
            let name = parts[2];
            (name.to_string(), domain)
        } else {
            ("@".to_string(), hostname.to_string())
        }
    }
}

impl DnsClient for GandiClient {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let (name, domain) = self.parse_hostname(hostname);
        
        let url = format!(
            "{}/v5/livedns/domains/{}/records/{}/A",
            self.server, domain, name
        );

        let body = json!({
            "rrset_values": [ip.to_string()],
            "rrset_ttl": 300
        });

        log::info!("Updating {} with Gandi", hostname);

        let response = minreq::put(&url)
            .with_header("Authorization", format!("Apikey {}", self.api_key))
            .with_header("Content-Type", "application/json")
            .with_json(&body)?
            .send()?;

        let status_code = response.status_code;

        if status_code == 200 || status_code == 201 {
            log::info!("DNS record for {} successfully updated to {}", hostname, ip);
            Ok(())
        } else {
            let body = response.as_str().unwrap_or("unknown error");
            Err(format!("Gandi API error ({}): {}", status_code, body).into())
        }
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.api_key.is_empty() {
            return Err("API key is required for Gandi".into());
        }
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "Gandi"
    }
}
