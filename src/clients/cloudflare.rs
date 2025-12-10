use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;
use serde_json::json;

pub struct CloudflareClient {
    login: String,
    password: String,
    zone: String,
    server: String,
    ttl: u32,
}

impl CloudflareClient {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let login = config.login.as_ref()
            .ok_or("login is required for Cloudflare (email or 'token')")?
            .clone();
        let password = config.password.as_ref()
            .ok_or("password is required for Cloudflare (API token or global API key)")?
            .clone();
        let zone = config.zone.as_ref()
            .ok_or("zone is required for Cloudflare (e.g., example.com)")?
            .clone();
        let server = config.server.as_ref()
            .map(|s| s.clone())
            .unwrap_or_else(|| "api.cloudflare.com/client/v4".to_string());
        let ttl = config.ttl.unwrap_or(1);

        Ok(CloudflareClient {
            login,
            password,
            zone,
            server,
            ttl,
        })
    }

    fn get_zone_id(&self) -> Result<String, Box<dyn Error>> {
        log::info!("Getting Cloudflare Zone ID for zone: {}", self.zone);

        let url = format!("https://{}/zones/?name={}", self.server, self.zone);
        
        let mut request = minreq::get(&url)
            .with_header("Content-Type", "application/json");
        
        // ddclient authentication: login=token uses Bearer, otherwise X-Auth-Email/Key
        if self.login == "token" {
            request = request.with_header("Authorization", format!("Bearer {}", self.password));
        } else {
            request = request
                .with_header("X-Auth-Email", &self.login)
                .with_header("X-Auth-Key", &self.password);
        }

        let res = request.send()?;
        let json: serde_json::Value = res.json()?;
        
        if !json["success"].as_bool().unwrap_or(false) {
            log::error!("Error getting zone ID: {}", json);
            return Err("Error getting zone ID".into());
        }

        let zone_id = json["result"][0]["id"]
            .as_str()
            .ok_or("No zone found")?
            .to_string();

        log::info!("Zone ID is {}", zone_id);
        Ok(zone_id)
    }

    fn get_record_id(&self, zone_id: &str, hostname: &str) -> Result<String, Box<dyn Error>> {
        log::info!("Fetching DNS record for: {}", hostname);

        let url = format!(
            "https://{}/zones/{}/dns_records?type=A&name={}",
            self.server, zone_id, hostname
        );

        let mut request = minreq::get(&url)
            .with_header("Content-Type", "application/json");
        
        if self.login == "token" {
            request = request.with_header("Authorization", format!("Bearer {}", self.password));
        } else {
            request = request
                .with_header("X-Auth-Email", &self.login)
                .with_header("X-Auth-Key", &self.password);
        }

        let res = request.send()?;
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
        let zone_id = self.get_zone_id()?;
        let record_id = self.get_record_id(&zone_id, hostname)?;

        let body = json!({
            "type": "A",
            "name": hostname,
            "content": ip.to_string(),
            "ttl": self.ttl,
        });

        let url = format!(
            "https://{}/zones/{}/dns_records/{}",
            self.server, zone_id, record_id
        );

        let mut request = minreq::put(&url)
            .with_header("Content-Type", "application/json")
            .with_json(&body)?;
        
        if self.login == "token" {
            request = request.with_header("Authorization", format!("Bearer {}", self.password));
        } else {
            request = request
                .with_header("X-Auth-Email", &self.login)
                .with_header("X-Auth-Key", &self.password);
        }

        let update_res = request.send()?;

        let update_json: serde_json::Value = update_res.json()?;
        
        if !update_json["success"].as_bool().unwrap_or(false) {
            log::error!("Failed to update DNS record: {}", update_json);
            return Err("Failed to update DNS record".into());
        }

        log::info!("DNS Record for {} successfully updated to IP: {}", hostname, ip);
        Ok(())
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.login.is_empty() {
            return Err("login is required for Cloudflare (email or 'token')".into());
        }
        if self.password.is_empty() {
            return Err("password is required for Cloudflare (API token or global API key)".into());
        }
        if self.zone.is_empty() {
            return Err("zone is required for Cloudflare (e.g., example.com)".into());
        }
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "Cloudflare"
    }
}
