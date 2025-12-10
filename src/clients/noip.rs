use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;
use base64::{Engine as _, engine::general_purpose};

/// No-IP client - compatible with DynDNS2 but with No-IP specifics
#[derive(Debug)]
pub struct NoIpClient {
    username: String,
    password: String,
    server: String,
}

impl NoIpClient {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let username = config.login.as_ref()
            .ok_or("username is required for No-IP")?
            .clone();
        let password = config.password.as_ref()
            .ok_or("password is required for No-IP")?
            .clone();
        
        let server = config.server.as_ref()
            .cloned()
            .unwrap_or_else(|| "https://dynupdate.no-ip.com".to_string());

        Ok(NoIpClient {
            username,
            password,
            server,
        })
    }
}

impl DnsClient for NoIpClient {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let url = format!(
            "{}/nic/update?hostname={}&myip={}",
            self.server, hostname, ip
        );

        log::info!("Updating {} with No-IP", hostname);

        let response = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .with_header("Authorization", format!("Basic {}", 
                general_purpose::STANDARD.encode(format!("{}:{}", self.username, self.password))))
            .send()?;

        let body = response.as_str()?.trim();
        let status = body.split_whitespace().next().unwrap_or("");

        match status {
            "good" | "nochg" => {
                log::info!("DNS record for {} successfully updated to {}", hostname, ip);
                Ok(())
            }
            "badauth" => Err("Bad authentication".into()),
            "nohost" => Err("Hostname doesn't exist".into()),
            "badagent" => Err("Client disabled - contact No-IP".into()),
            "abuse" => Err("Username blocked for abuse".into()),
            "911" => Err("Server error - try again later".into()),
            _ => Err(format!("Unknown response: {}", body).into()),
        }
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.username.is_empty() {
            return Err("username is required for No-IP".into());
        }
        if self.password.is_empty() {
            return Err("password is required for No-IP".into());
        }
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "No-IP"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> Config {
        Config {
            protocol: Some("noip".to_string()),
            login: Some("testuser".to_string()),
            password: Some("testpass".to_string()),
            host: Some("myhost.no-ip.com".to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn test_noip_client_creation() {
        let config = create_test_config();
        let client = NoIpClient::new(&config);
        
        assert!(client.is_ok());
        let client = client.unwrap();
        assert_eq!(client.username, "testuser");
        assert_eq!(client.password, "testpass");
        assert_eq!(client.server, "https://dynupdate.no-ip.com");
    }

    #[test]
    fn test_noip_custom_server() {
        let config = Config {
            protocol: Some("noip".to_string()),
            login: Some("testuser".to_string()),
            password: Some("testpass".to_string()),
            server: Some("https://custom.no-ip.com".to_string()),
            ..Default::default()
        };
        
        let client = NoIpClient::new(&config);
        assert!(client.is_ok());
        assert_eq!(client.unwrap().server, "https://custom.no-ip.com");
    }

    #[test]
    fn test_noip_missing_username() {
        let config = Config {
            protocol: Some("noip".to_string()),
            login: None,
            password: Some("testpass".to_string()),
            ..Default::default()
        };
        
        let result = NoIpClient::new(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("username is required"));
    }

    #[test]
    fn test_noip_missing_password() {
        let config = Config {
            protocol: Some("noip".to_string()),
            login: Some("testuser".to_string()),
            password: None,
            ..Default::default()
        };
        
        let result = NoIpClient::new(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("password is required"));
    }

    #[test]
    fn test_noip_validate_config() {
        let config = create_test_config();
        let client = NoIpClient::new(&config).unwrap();
        
        assert!(client.validate_config().is_ok());
    }

    #[test]
    fn test_noip_provider_name() {
        let config = create_test_config();
        let client = NoIpClient::new(&config).unwrap();
        
        assert_eq!(client.provider_name(), "No-IP");
    }

    #[test]
    fn test_noip_url_construction() {
        let config = create_test_config();
        let client = NoIpClient::new(&config).unwrap();
        
        let hostname = "myhost.no-ip.com";
        let ip = "203.0.113.1";
        let expected_url = format!(
            "{}/nic/update?hostname={}&myip={}",
            client.server, hostname, ip
        );
        
        assert_eq!(expected_url, "https://dynupdate.no-ip.com/nic/update?hostname=myhost.no-ip.com&myip=203.0.113.1");
    }
}
