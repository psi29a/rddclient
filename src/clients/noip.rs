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
    /// Creates a NoIpClient from a configuration, requiring username and password and using
    /// "https://dynupdate.no-ip.com" as the default update server when none is provided.
    ///
    /// Returns an error if `config.login` or `config.password` is missing.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let cfg = Config {
    ///     login: Some("user".into()),
    ///     password: Some("pass".into()),
    ///     server: None,
    ///     ..Default::default()
    /// };
    /// let client = NoIpClient::new(&cfg).unwrap();
    /// assert_eq!(client.provider_name(), "No-IP");
    /// ```
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
    /// Update the DNS record for `hostname` at the configured No‑IP server to the given `ip`.
    ///
    /// On success returns `Ok(())`. On error returns a boxed `Error` describing the failure; common failure reasons include authentication errors, unknown hostname, client disabled/blocked responses, server errors, or an unexpected response body.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::net::IpAddr;
    /// # use crate::clients::noip::NoIpClient;
    /// # // Create a NoIpClient via NoIpClient::new(...) in real code.
    /// let ip: IpAddr = "1.2.3.4".parse().unwrap();
    /// // client.update_record("host.example.com", ip).unwrap();
    /// ```
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

    /// Ensures the client has the required credentials for No‑IP.
    ///
    /// Returns `Ok(())` when both username and password are non-empty, or an `Err` with a descriptive message otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = NoIpClient { username: "user".into(), password: "pass".into(), server: "https://dynupdate.no-ip.com".into() };
    /// assert!(client.validate_config().is_ok());
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.username.is_empty() {
            return Err("username is required for No-IP".into());
        }
        if self.password.is_empty() {
            return Err("password is required for No-IP".into());
        }
        Ok(())
    }

    /// Provider identifier for this DNS client.
    ///
    /// This returns the human-readable name of the provider implemented by this client.
    ///
    /// # Examples
    ///
    /// ```
    /// // Given a configured `NoIpClient`:
    /// // let client = NoIpClient::new(&config).unwrap();
    /// // assert_eq!(client.provider_name(), "No-IP");
    /// ```
    fn provider_name(&self) -> &str {
        "No-IP"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a Config populated with typical test values for the No-IP client.
    ///
    /// The returned `Config` contains:
    /// - `protocol = Some("noip")`
    /// - `login = Some("testuser")`
    /// - `password = Some("testpass")`
    /// - `host = Some("myhost.no-ip.com")`
    /// All other fields are left as their `Default` values.
    ///
    /// # Examples
    ///
    /// ```
    /// let cfg = create_test_config();
    /// assert_eq!(cfg.protocol.as_deref(), Some("noip"));
    /// assert_eq!(cfg.login.as_deref(), Some("testuser"));
    /// assert_eq!(cfg.password.as_deref(), Some("testpass"));
    /// assert_eq!(cfg.host.as_deref(), Some("myhost.no-ip.com"));
    /// ```
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