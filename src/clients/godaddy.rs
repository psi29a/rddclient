use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;
use serde_json::json;

/// GoDaddy client - https://developer.godaddy.com/doc/endpoint/domains
pub struct GoDaddyClient {
    api_key: String,
    api_secret: String,
    server: String,
}

impl GoDaddyClient {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let api_key = config.login.as_ref()
            .ok_or("username (API key) is required for GoDaddy")?
            .clone();
        let api_secret = config.password.as_ref()
            .ok_or("password (API secret) is required for GoDaddy")?
            .clone();
        
        let server = config.server.as_ref()
            .cloned()
            .unwrap_or_else(|| "https://api.godaddy.com".to_string());

        Ok(GoDaddyClient {
            api_key,
            api_secret,
            server,
        })
    }

    fn parse_hostname(&self, hostname: &str) -> (String, String) {
        // Split hostname into domain and record name
        // e.g., "www.example.com" -> ("www", "example.com")
        let parts: Vec<&str> = hostname.rsplitn(3, '.').collect();
        if parts.len() >= 3 {
            let domain = format!("{}.{}", parts[1], parts[0]);
            let name = parts[2];
            (name.to_string(), domain)
        } else {
            // Assume @ for root domain
            ("@".to_string(), hostname.to_string())
        }
    }
}

impl DnsClient for GoDaddyClient {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let (name, domain) = self.parse_hostname(hostname);
        
        // Determine record type based on IP version
        let record_type = match ip {
            IpAddr::V4(_) => "A",
            IpAddr::V6(_) => "AAAA",
        };
        
        let url = format!(
            "{}/v1/domains/{}/records/{}/{}",
            self.server, domain, record_type, name
        );

        let body = json!([{
            "data": ip.to_string(),
            "ttl": 600
        }]);

        log::info!("Updating {} with GoDaddy", hostname);

        let response = minreq::put(&url)
            .with_header("Authorization", format!("sso-key {}:{}", self.api_key, self.api_secret))
            .with_header("Content-Type", "application/json")
            .with_json(&body)?
            .send()?;

        let status_code = response.status_code;

        if status_code == 200 {
            log::info!("DNS record for {} successfully updated to {}", hostname, ip);
            Ok(())
        } else {
            let body = response.as_str().unwrap_or("unknown error");
            Err(format!("GoDaddy API error ({}): {}", status_code, body).into())
        }
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.api_key.is_empty() {
            return Err("API key (username) is required for GoDaddy".into());
        }
        if self.api_secret.is_empty() {
            return Err("API secret (password) is required for GoDaddy".into());
        }
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "GoDaddy"
    }
}
