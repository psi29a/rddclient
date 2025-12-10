use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;
use serde_json::json;

pub struct CloudflareClient {
    zone_id: String,
    api_token: String,
    ttl: u32,
}

impl CloudflareClient {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let zone_id = config.zone_id.as_ref()
            .ok_or("zone_id is required for Cloudflare")?
            .clone();
        let api_token = config.api_token.as_ref()
            .ok_or("api_token is required for Cloudflare")?
            .clone();
        let ttl = config.ttl.unwrap_or(1);

        Ok(CloudflareClient {
            zone_id,
            api_token,
            ttl,
        })
    }

    fn get_record_id(&self, hostname: &str) -> Result<String, Box<dyn Error>> {
        log::info!("Fetching DNS record for: {}", hostname);

        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records?type=A&name={}",
            self.zone_id, hostname
        );

        let res = minreq::get(&url)
            .with_header("Authorization", format!("Bearer {}", self.api_token))
            .with_header("Content-Type", "application/json")
            .send()?;

        let json: serde_json::Value = res.json()?;
        
        if !json["success"].as_bool().unwrap_or(false) {
            log::error!("Error getting DNS record info: {}", json);
            return Err("Error getting DNS record info".into());
        }

        let record_id = json["result"][0]["id"]
            .as_str()
            .ok_or("No DNS record found")?
            .to_string();

        log::info!("DNS Record ID for {} is {}", hostname, record_id);
        Ok(record_id)
    }
}

impl DnsClient for CloudflareClient {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let record_id = self.get_record_id(hostname)?;

        let body = json!({
            "type": "A",
            "name": hostname,
            "content": ip.to_string(),
            "ttl": self.ttl,
        });

        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
            self.zone_id, record_id
        );

        let update_res = minreq::put(&url)
            .with_header("Authorization", format!("Bearer {}", self.api_token))
            .with_header("Content-Type", "application/json")
            .with_json(&body)?
            .send()?;

        let update_json: serde_json::Value = update_res.json()?;
        
        if !update_json["success"].as_bool().unwrap_or(false) {
            log::error!("Failed to update DNS record: {}", update_json);
            return Err("Failed to update DNS record".into());
        }

        log::info!("DNS Record for {} successfully updated to IP: {}", hostname, ip);
        Ok(())
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.zone_id.is_empty() {
            return Err("zone_id is required for Cloudflare".into());
        }
        if self.api_token.is_empty() {
            return Err("api_token is required for Cloudflare".into());
        }
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "Cloudflare"
    }
}
