use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

pub struct HetznerClient {
    api_token: String,
    zone_id: String,
    server: String,
}

impl HetznerClient {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let api_token = config.password.as_ref()
            .or(config.password.as_ref())
            .ok_or("Hetzner requires API token (use password or api_token)")?
            .clone();
        let zone_id = config.zone.as_ref()
            .ok_or("Hetzner requires zone_id (domain name)")?
            .clone();
        let server = config.server.clone()
            .unwrap_or_else(|| "https://dns.hetzner.com".to_string());

        Ok(Self {
            api_token,
            zone_id,
            server,
        })
    }

    /// Extract subdomain from FQDN by stripping zone suffix
    /// e.g., "www.example.com" with zone "example.com" -> "www"
    fn extract_subdomain(&self, hostname: &str) -> String {
        let zone_suffix = format!(".{}", self.zone_id);
        if let Some(subdomain) = hostname.strip_suffix(&zone_suffix) {
            subdomain.to_string()
        } else if hostname == self.zone_id {
            // Apex domain
            "@".to_string()
        } else {
            // No match, use as-is
            hostname.to_string()
        }
    }

    fn get_record_id(&self, hostname: &str, record_type: &str) -> Result<String, Box<dyn Error>> {
        let url = format!("{}/records?zone_id={}", self.server, self.zone_id);
        
        let response = minreq::get(&url)
            .with_header("Auth-API-Token", &self.api_token)
            .send()?;

        if response.status_code != 200 {
            return Err(format!("Failed to fetch records: HTTP {}", response.status_code).into());
        }

        let json: serde_json::Value = response.json()?;
        
        // Strip zone suffix to get subdomain for comparison
        let subdomain = self.extract_subdomain(hostname);
        
        if let Some(records) = json["records"].as_array() {
            for record in records {
                // Compare against API's relative names (e.g., "www" not "www.example.com")
                if record["name"].as_str() == Some(&subdomain) 
                    && record["type"].as_str() == Some(record_type) {
                    if let Some(id) = record["id"].as_str() {
                        log::debug!("Found record ID {} for {} (subdomain: {})", id, hostname, subdomain);
                        return Ok(id.to_string());
                    }
                }
            }
        }

        Err(format!("Record {} (subdomain: {}) not found", hostname, subdomain).into())
    }
}

impl DnsClient for HetznerClient {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        // Determine record type based on IP version
        let record_type = match ip {
            IpAddr::V4(_) => "A",
            IpAddr::V6(_) => "AAAA",
        };
        
        log::info!("Fetching {} record ID for {}", record_type, hostname);
        let record_id = self.get_record_id(hostname, record_type)?;
        
        let url = format!("{}/records/{}", self.server, record_id);
        
        // Use subdomain (not FQDN) in API call
        let subdomain = self.extract_subdomain(hostname);
        
        let payload = serde_json::json!({
            "value": ip.to_string(),
            "ttl": 60,
            "type": record_type,
            "name": subdomain,
            "zone_id": self.zone_id
        });

        log::info!("Updating {} to {}", hostname, ip);
        
        let response = minreq::put(&url)
            .with_header("Auth-API-Token", &self.api_token)
            .with_header("Content-Type", "application/json")
            .with_json(&payload)?
            .send()?;

        if response.status_code == 200 {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else {
            Err(format!("Failed to update record: HTTP {}", response.status_code).into())
        }
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.api_token.is_empty() {
            return Err("Hetzner API token cannot be empty".into());
        }
        if self.zone_id.is_empty() {
            return Err("Hetzner zone_id cannot be empty".into());
        }
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "Hetzner"
    }
}
