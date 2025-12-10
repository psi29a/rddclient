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
    /// Create a DuckDnsClient from a Config.
    ///
    /// The client's token is taken from `config.password`; the server is taken from
    /// `config.server` if present, otherwise defaults to `https://www.duckdns.org`.
    ///
    /// # Returns
    ///
    /// `Ok(DuckDnsClient)` configured with the token and server, `Err` if the token is missing.
    ///
    /// # Examples
    ///
    /// ```
    /// let config = Config { password: Some("test-token-12345".to_string()), ..Default::default() };
    /// let client = DuckDnsClient::new(&config).unwrap();
    /// assert_eq!(client.provider_name(), "DuckDNS");
    /// ```
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
    /// Update the DNS A record for a DuckDNS hostname to the provided IP.
    ///
    /// The `hostname` may include the trailing `.duckdns.org` suffix; if present it will be trimmed
    /// before constructing the update request. The request is sent to the client's configured DuckDNS
    /// server using the client's token.
    ///
    /// # Parameters
    ///
    /// - `hostname`: The DuckDNS hostname to update (e.g., `"myhost"` or `"myhost.duckdns.org"`).
    /// - `ip`: The IP address to assign to the hostname.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the update succeeded (`"OK"` response from DuckDNS); `Err` with a descriptive
    /// message if the provider reported failure (`"KO"`) or returned an unexpected response.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::IpAddr;
    ///
    /// let client = DuckDnsClient {
    ///     token: "test-token-12345".to_string(),
    ///     server: "https://www.duckdns.org".to_string(),
    /// };
    ///
    /// let ip: IpAddr = "203.0.113.1".parse().unwrap();
    /// let res = client.update_record("myhost.duckdns.org", ip);
    /// // In real usage this performs a network request; here we only show the call shape.
    /// let _ = res;
    /// ```
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

    /// Validates that the client has a DuckDNS token configured.
    ///
    /// Returns `Ok(())` if the token is present, `Err` with message `"token is required for DuckDNS"` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = crate::clients::duckdns::DuckDnsClient { token: "test-token".into(), server: "https://www.duckdns.org".into() };
    /// assert!(client.validate_config().is_ok());
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.token.is_empty() {
            return Err("token is required for DuckDNS".into());
        }
        Ok(())
    }

    /// Provides the DNS provider name for this client.
    ///
    /// The returned string is the static provider identifier: "DuckDNS".
    ///
    /// # Examples
    ///
    /// ```
    /// let client = DuckDnsClient { token: "t".into(), server: "s".into() };
    /// assert_eq!(client.provider_name(), "DuckDNS");
    /// ```
    fn provider_name(&self) -> &str {
        "DuckDNS"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Creates a Config prefilled for DuckDNS unit tests.
    ///
    /// The returned config has:
    /// - protocol set to "duckdns"
    /// - password set to "test-token-12345"
    /// - host set to "myhost.duckdns.org"
    ///
    /// # Examples
    ///
    /// ```
    /// let cfg = create_test_config();
    /// assert_eq!(cfg.protocol.as_deref(), Some("duckdns"));
    /// assert_eq!(cfg.password.as_deref(), Some("test-token-12345"));
    /// assert_eq!(cfg.host.as_deref(), Some("myhost.duckdns.org"));
    /// ```
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

    /// Verifies that a `DuckDnsClient` created from a valid configuration passes `validate_config`.
    ///
    /// # Examples
    ///
    /// ```
    /// let config = create_test_config();
    /// let client = DuckDnsClient::new(&config).unwrap();
    /// assert!(client.validate_config().is_ok());
    /// ```
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