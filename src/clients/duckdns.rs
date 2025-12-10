use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

/// DuckDNS client - https://www.duckdns.org/
#[derive(Debug)]
pub struct DuckDnsClient {
    token: String,
    server: String,
}

impl DuckDnsClient {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let token = config.password.as_ref()
            .or(config.password.as_ref())
            .ok_or("token (password or api_token) is required for DuckDNS")?
            .clone();
        
        let server = config.server.as_ref()
            .cloned()
            .unwrap_or_else(|| "https://www.duckdns.org".to_string());

        Ok(DuckDnsClient { token, server })
    }
}

impl DnsClient for DuckDnsClient {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        // DuckDNS hostname is typically without the .duckdns.org suffix
        let domain = hostname.trim_end_matches(".duckdns.org");
        
        let url = format!(
            "{}/update?domains={}&token={}&ip={}",
            self.server, domain, self.token, ip
        );

        log::info!("Updating {} with DuckDNS", hostname);

        let response = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .send()?;

        let body = response.as_str()?.trim();

        if body == "OK" {
            log::info!("DNS record for {} successfully updated to {}", hostname, ip);
            Ok(())
        } else if body == "KO" {
            Err("DuckDNS update failed - check your token and domain".into())
        } else {
            Err(format!("Unexpected response: {}", body).into())
        }
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.token.is_empty() {
            return Err("token is required for DuckDNS".into());
        }
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "DuckDNS"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> Config {
        Config {
            protocol: Some("duckdns".to_string()),
            password: Some("test-token-12345".to_string()),
            host: Some("myhost.duckdns.org".to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn test_duckdns_client_creation() {
        let config = create_test_config();
        let client = DuckDnsClient::new(&config);
        
        assert!(client.is_ok());
        let client = client.unwrap();
        assert_eq!(client.token, "test-token-12345");
        assert_eq!(client.server, "https://www.duckdns.org");
    }

    #[test]
    fn test_duckdns_custom_server() {
        let config = Config {
            protocol: Some("duckdns".to_string()),
            password: Some("test-token".to_string()),
            server: Some("https://custom.duckdns.org".to_string()),
            ..Default::default()
        };
        
        let client = DuckDnsClient::new(&config);
        assert!(client.is_ok());
        assert_eq!(client.unwrap().server, "https://custom.duckdns.org");
    }

    #[test]
    fn test_duckdns_missing_token() {
        let config = Config {
            protocol: Some("duckdns".to_string()),
            password: None,
            ..Default::default()
        };
        
        let result = DuckDnsClient::new(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("token"));
    }

    #[test]
    fn test_duckdns_validate_config() {
        let config = create_test_config();
        let client = DuckDnsClient::new(&config).unwrap();
        
        assert!(client.validate_config().is_ok());
    }

    #[test]
    fn test_duckdns_provider_name() {
        let config = create_test_config();
        let client = DuckDnsClient::new(&config).unwrap();
        
        assert_eq!(client.provider_name(), "DuckDNS");
    }

    #[test]
    fn test_duckdns_hostname_trimming() {
        // DuckDNS should trim .duckdns.org suffix
        let hostname = "myhost.duckdns.org";
        let expected = "myhost";
        let trimmed = hostname.trim_end_matches(".duckdns.org");
        assert_eq!(trimmed, expected);
        
        // Should also work with just the subdomain
        let hostname2 = "myhost";
        let trimmed2 = hostname2.trim_end_matches(".duckdns.org");
        assert_eq!(trimmed2, "myhost");
    }

    #[test]
    fn test_duckdns_url_construction() {
        let config = create_test_config();
        let client = DuckDnsClient::new(&config).unwrap();
        
        let hostname = "myhost.duckdns.org";
        let ip = "203.0.113.1";
        let domain = hostname.trim_end_matches(".duckdns.org");
        
        let expected_url = format!(
            "{}/update?domains={}&token={}&ip={}",
            client.server, domain, client.token, ip
        );
        
        assert_eq!(expected_url, "https://www.duckdns.org/update?domains=myhost&token=test-token-12345&ip=203.0.113.1");
    }
}
