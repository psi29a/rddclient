use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;
use serde_json::json;

/// DigitalOcean client - https://docs.digitalocean.com/reference/api/api-reference/#tag/Domain-Records
pub struct DigitalOceanClient {
    token: String,
    server: String,
}

impl DigitalOceanClient {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let token = config.password.as_ref()
            .or(config.password.as_ref())
            .ok_or("api_token or password is required for DigitalOcean")?
            .clone();
        
        let server = config.server.as_ref()
            .cloned()
            .unwrap_or_else(|| "https://api.digitalocean.com".to_string());

        Ok(DigitalOceanClient { token, server })
    }

    fn parse_hostname(&self, hostname: &str) -> (String, String) {
        // Split hostname into record name and domain
        let parts: Vec<&str> = hostname.rsplitn(3, '.').collect();
        if parts.len() >= 3 {
            let domain = format!("{}.{}", parts[1], parts[0]);
            let name = parts[2];
            (name.to_string(), domain)
        } else {
            ("@".to_string(), hostname.to_string())
        }
    }

    fn get_record_id(&self, domain: &str, name: &str, record_type: &str) -> Result<u64, Box<dyn Error>> {
        let url = format!("{}/v2/domains/{}/records", self.server, domain);

        let response = minreq::get(&url)
            .with_header("Authorization", format!("Bearer {}", self.token))
            .with_header("Content-Type", "application/json")
            .send()?;

        let json: serde_json::Value = response.json()?;
        
        if let Some(records) = json["domain_records"].as_array() {
            for record in records {
                if record["type"] == record_type && record["name"] == name {
                    if let Some(id) = record["id"].as_u64() {
                        return Ok(id);
                    }
                }
            }
        }

        Err(format!("No {} record found for {}.{}", record_type, name, domain).into())
    }
}

impl DnsClient for DigitalOceanClient {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let (name, domain) = self.parse_hostname(hostname);
        
        // Determine record type based on IP version
        let record_type = match ip {
            IpAddr::V4(_) => "A",
            IpAddr::V6(_) => "AAAA",
        };
        
        log::info!("Updating {} with DigitalOcean ({})", hostname, record_type);
        
        let record_id = self.get_record_id(&domain, &name, record_type)?;
        
        let url = format!("{}/v2/domains/{}/records/{}", self.server, domain, record_id);

        let body = json!({
            "data": ip.to_string()
        });

        let response = minreq::put(&url)
            .with_header("Authorization", format!("Bearer {}", self.token))
            .with_header("Content-Type", "application/json")
            .with_json(&body)?
            .send()?;

        let status_code = response.status_code;

        if status_code == 200 {
            log::info!("DNS record for {} successfully updated to {}", hostname, ip);
            Ok(())
        } else {
            let body = response.as_str().unwrap_or("unknown error");
            Err(format!("DigitalOcean API error ({}): {}", status_code, body).into())
        }
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.token.is_empty() {
            return Err("API token is required for DigitalOcean".into());
        }
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "DigitalOcean"
    }
}
