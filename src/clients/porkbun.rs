use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;
use serde_json::json;

/// Porkbun client - https://porkbun.com/api/json/v3/documentation
pub struct PorkbunClient {
    api_key: String,
    secret_key: String,
    server: String,
}

impl PorkbunClient {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let api_key = config.login.as_ref()
            .ok_or("username (API key) is required for Porkbun")?
            .clone();
        let secret_key = config.password.as_ref()
            .ok_or("password (secret key) is required for Porkbun")?
            .clone();
        
        let server = config.server.as_ref()
            .cloned()
            .unwrap_or_else(|| "https://porkbun.com/api/json/v3".to_string());

        Ok(PorkbunClient {
            api_key,
            secret_key,
            server,
        })
    }

    fn parse_hostname(&self, hostname: &str) -> (String, String) {
        let parts: Vec<&str> = hostname.rsplitn(3, '.').collect();
        if parts.len() >= 3 {
            let domain = format!("{}.{}", parts[1], parts[0]);
            let name = parts[2];
            (name.to_string(), domain)
        } else {
            ("".to_string(), hostname.to_string())
        }
    }
}

impl DnsClient for PorkbunClient {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let (name, domain) = self.parse_hostname(hostname);
        
        let url = format!("{}/dns/editByNameType/{}/A", self.server, domain);
        
        let subdomain = if name.is_empty() { None } else { Some(name.as_str()) };

        let mut body = json!({
            "apikey": self.api_key,
            "secretapikey": self.secret_key,
            "content": ip.to_string(),
            "ttl": "600"
        });

        if let Some(sub) = subdomain {
            body["name"] = json!(sub);
        }

        log::info!("Updating {} with Porkbun", hostname);

        let response = minreq::post(&url)
            .with_header("Content-Type", "application/json")
            .with_json(&body)?
            .send()?;

        let json: serde_json::Value = response.json()?;

        if json["status"] == "SUCCESS" {
            log::info!("DNS record for {} successfully updated to {}", hostname, ip);
            Ok(())
        } else {
            let message = json["message"].as_str().unwrap_or("unknown error");
            Err(format!("Porkbun API error: {}", message).into())
        }
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.api_key.is_empty() {
            return Err("API key (username) is required for Porkbun".into());
        }
        if self.secret_key.is_empty() {
            return Err("Secret key (password) is required for Porkbun".into());
        }
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "Porkbun"
    }
}
