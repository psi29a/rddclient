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

    fn get_record_id(&self, hostname: &str, record_type: &str) -> Result<String, Box<dyn Error>> {
        let url = format!("{}/records?zone_id={}", self.server, self.zone_id);
        
        let response = minreq::get(&url)
            .with_header("Auth-API-Token", &self.api_token)
            .send()?;

        if response.status_code != 200 {
            return Err(format!("Failed to fetch records: HTTP {}", response.status_code).into());
        }

        let json: serde_json::Value = response.json()?;
        
        if let Some(records) = json["records"].as_array() {
            for record in records {
                if record["name"].as_str() == Some(hostname) 
                    && record["type"].as_str() == Some(record_type) {
                    if let Some(id) = record["id"].as_str() {
                        return Ok(id.to_string());
                    }
                }
            }
        }

        Err(format!("Record {} not found", hostname).into())
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
        
        let payload = serde_json::json!({
            "value": ip.to_string(),
            "ttl": 60,
            "type": record_type,
            "name": hostname,
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
