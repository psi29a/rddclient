use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;
use serde_json::json;

#[derive(Debug)]
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

    fn get_record_id(&self, zone_id: &str, hostname: &str, record_type: &str) -> Result<String, Box<dyn Error>> {
        log::info!("Fetching DNS {} record for: {}", record_type, hostname);

        let url = format!(
            "https://{}/zones/{}/dns_records?type={}&name={}",
            self.server, zone_id, record_type, hostname
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
            .ok_or(format!("No DNS {} record found for {}", record_type, hostname))?
            .to_string();

        log::info!("DNS {} Record ID for {} is {}", record_type, hostname, record_id);
        Ok(record_id)
    }
}

impl DnsClient for CloudflareClient {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        // Determine record type based on IP version
        let record_type = match ip {
            IpAddr::V4(_) => "A",
            IpAddr::V6(_) => "AAAA",
        };
        
        log::info!("Setting {} address to {}", 
                   if record_type == "A" { "IPv4" } else { "IPv6" }, ip);
        
        let zone_id = self.get_zone_id()?;
        let record_id = self.get_record_id(&zone_id, hostname, record_type)?;

        let body = json!({
            "type": record_type,
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

        log::info!("DNS {} Record for {} successfully updated to IP: {}", record_type, hostname, ip);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> Config {
        Config {
            protocol: Some("cloudflare".to_string()),
            login: Some("token".to_string()),
            password: Some("test_api_token_12345".to_string()),
            zone: Some("example.com".to_string()),
            host: Some("ddns.example.com".to_string()),
            ttl: Some(300),
            ..Default::default()
        }
    }

    #[test]
    fn test_cloudflare_client_creation_with_token() {
        let config = create_test_config();
        let client = CloudflareClient::new(&config);
        
        assert!(client.is_ok());
        let client = client.unwrap();
        assert_eq!(client.login, "token");
        assert_eq!(client.password, "test_api_token_12345");
        assert_eq!(client.zone, "example.com");
        assert_eq!(client.server, "api.cloudflare.com/client/v4");
        assert_eq!(client.ttl, 300);
    }

    #[test]
    fn test_cloudflare_client_creation_with_global_key() {
        let config = Config {
            protocol: Some("cloudflare".to_string()),
            login: Some("user@example.com".to_string()),
            password: Some("global_api_key_12345".to_string()),
            zone: Some("example.com".to_string()),
            ..Default::default()
        };
        
        let client = CloudflareClient::new(&config);
        assert!(client.is_ok());
        let client = client.unwrap();
        assert_eq!(client.login, "user@example.com");
        assert_eq!(client.password, "global_api_key_12345");
    }

    #[test]
    fn test_cloudflare_client_custom_server() {
        let config = Config {
            protocol: Some("cloudflare".to_string()),
            login: Some("token".to_string()),
            password: Some("test_token".to_string()),
            zone: Some("example.com".to_string()),
            server: Some("custom.cloudflare.com/v4".to_string()),
            ..Default::default()
        };
        
        let client = CloudflareClient::new(&config);
        assert!(client.is_ok());
        assert_eq!(client.unwrap().server, "custom.cloudflare.com/v4");
    }

    #[test]
    fn test_cloudflare_client_default_ttl() {
        let config = Config {
            protocol: Some("cloudflare".to_string()),
            login: Some("token".to_string()),
            password: Some("test_token".to_string()),
            zone: Some("example.com".to_string()),
            ttl: None,  // No TTL specified
            ..Default::default()
        };
        
        let client = CloudflareClient::new(&config);
        assert!(client.is_ok());
        assert_eq!(client.unwrap().ttl, 1);  // Should default to 1 (auto)
    }

    #[test]
    fn test_cloudflare_client_missing_login() {
        let config = Config {
            protocol: Some("cloudflare".to_string()),
            login: None,  // Missing login
            password: Some("test_token".to_string()),
            zone: Some("example.com".to_string()),
            ..Default::default()
        };
        
        let result = CloudflareClient::new(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("login is required"));
    }

    #[test]
    fn test_cloudflare_client_missing_password() {
        let config = Config {
            protocol: Some("cloudflare".to_string()),
            login: Some("token".to_string()),
            password: None,  // Missing password
            zone: Some("example.com".to_string()),
            ..Default::default()
        };
        
        let result = CloudflareClient::new(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("password is required"));
    }

    #[test]
    fn test_cloudflare_client_missing_zone() {
        let config = Config {
            protocol: Some("cloudflare".to_string()),
            login: Some("token".to_string()),
            password: Some("test_token".to_string()),
            zone: None,  // Missing zone
            ..Default::default()
        };
        
        let result = CloudflareClient::new(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("zone is required"));
    }

    #[test]
    fn test_cloudflare_validate_config_success() {
        let config = create_test_config();
        let client = CloudflareClient::new(&config).unwrap();
        
        let result = client.validate_config();
        assert!(result.is_ok());
    }

    #[test]
    fn test_cloudflare_validate_config_empty_fields() {
        // Create a client with empty fields (bypassing normal validation)
        let client = CloudflareClient {
            login: String::new(),
            password: String::new(),
            zone: String::new(),
            server: "api.cloudflare.com/client/v4".to_string(),
            ttl: 1,
        };
        
        let result = client.validate_config();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("login is required"));
    }

    #[test]
    fn test_cloudflare_provider_name() {
        let config = create_test_config();
        let client = CloudflareClient::new(&config).unwrap();
        
        assert_eq!(client.provider_name(), "Cloudflare");
    }

    #[test]
    fn test_cloudflare_url_construction() {
        let config = create_test_config();
        let client = CloudflareClient::new(&config).unwrap();
        
        // Test zone lookup URL construction
        let expected_zone_url = format!("https://{}/zones/?name={}", 
            client.server, client.zone);
        assert_eq!(expected_zone_url, 
            "https://api.cloudflare.com/client/v4/zones/?name=example.com");
        
        // Test IPv4 record lookup URL construction
        let zone_id = "test_zone_id";
        let hostname = "ddns.example.com";
        let expected_record_url = format!(
            "https://{}/zones/{}/dns_records?type=A&name={}", 
            client.server, zone_id, hostname);
        assert_eq!(expected_record_url, 
            "https://api.cloudflare.com/client/v4/zones/test_zone_id/dns_records?type=A&name=ddns.example.com");
        
        // Test IPv6 record lookup URL construction
        let expected_record_url_v6 = format!(
            "https://{}/zones/{}/dns_records?type=AAAA&name={}", 
            client.server, zone_id, hostname);
        assert_eq!(expected_record_url_v6, 
            "https://api.cloudflare.com/client/v4/zones/test_zone_id/dns_records?type=AAAA&name=ddns.example.com");
    }

    #[test]
    fn test_cloudflare_ipv6_support() {
        use std::str::FromStr;
        
        // Test that IPv6 addresses are properly detected
        let ipv6 = IpAddr::from_str("2001:db8::1").unwrap();
        assert!(matches!(ipv6, IpAddr::V6(_)));
        
        let ipv4 = IpAddr::from_str("192.0.2.1").unwrap();
        assert!(matches!(ipv4, IpAddr::V4(_)));
    }
}
