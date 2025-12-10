use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;
use base64::{Engine as _, engine::general_purpose};

/// DynDNS2 protocol client
/// This protocol is supported by many providers including:
/// - DynDNS
/// - No-IP
/// - DNSdynamic
/// - DuckDNS
/// - Many others
#[derive(Debug)]
pub struct DynDns2Client {
    server: String,
    username: String,
    password: String,
    script: String,
}

impl DynDns2Client {
    /// Constructs a `DynDns2Client` from the provided configuration.
    ///
    /// The `login` and `password` fields of `config` are required; this function returns
    /// an `Err` if either is missing. If `server` is not provided, the default
    /// "https://members.dyndns.org" is used and the update script path is set to "/nic/update".
    ///
    /// # Examples
    ///
    /// ```
    /// let config = crate::Config {
    ///     login: Some("user".to_string()),
    ///     password: Some("pass".to_string()),
    ///     server: None,
    ///     ..Default::default()
    /// };
    /// let client = crate::clients::dyndns2::DynDns2Client::new(&config).unwrap();
    /// assert_eq!(client.provider_name(), "DynDNS2");
    /// ```
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let username = config.login.as_ref()
            .ok_or("username is required for DynDNS2")?
            .clone();
        let password = config.password.as_ref()
            .ok_or("password is required for DynDNS2")?
            .clone();
        
        // Default server and script path for standard DynDNS2 protocol
        let server = config.server.as_ref()
            .cloned()
            .unwrap_or_else(|| "https://members.dyndns.org".to_string());
        let script = "/nic/update".to_string();

        Ok(DynDns2Client {
            server,
            username,
            password,
            script,
        })
    }
}

impl DnsClient for DynDns2Client {
    /// Updates the DNS record for `hostname` to the provided `ip` using the DynDNS2 protocol.
    ///
    /// Sends an authenticated HTTP GET to the provider's DynDNS2 update endpoint and interprets
    /// the provider response. On success this will log and return Ok(()); on failure it returns
    /// an error describing the HTTP or protocol-level failure (for example `badauth`, `nohost`,
    /// `notfqdn`, or an unknown response body).
    ///
    /// # Arguments
    ///
    /// * `hostname` - The DNS host name to update (e.g. "host.example.com").
    /// * `ip` - The IP address to assign to `hostname`.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the update succeeded or the provider reported no change; `Err(...)` if the HTTP
    /// response code is not 200 or the provider returned an error status.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::IpAddr;
    /// // Assuming `client` is an initialized DynDns2Client:
    /// // let client = DynDns2Client::new(&config).unwrap();
    /// // client.update_record("ddns.example.com", "203.0.113.5".parse::<IpAddr>().unwrap()).unwrap();
    /// ```
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let url = format!(
            "{}{}?hostname={}&myip={}",
            self.server, self.script, hostname, ip
        );

        log::info!("Updating {} with DynDNS2 protocol", hostname);

        let response = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .with_header("Authorization", format!("Basic {}", 
                general_purpose::STANDARD.encode(format!("{}:{}", self.username, self.password))))
            .send()?;

        let status_code = response.status_code;
        let body = response.as_str()?.trim();

        log::debug!("Response status: {}, body: {}", status_code, body);

        if status_code != 200 {
            return Err(format!("HTTP error: {}", status_code).into());
        }

        // Parse DynDNS2 response
        // Format: "status [ip]" where status is one of:
        // - good: Update successful
        // - nochg: No change needed (IP is the same)
        // - badauth: Bad authorization (username/password)
        // - notfqdn: Not a fully-qualified domain name
        // - nohost: Hostname doesn't exist
        // - !yours: Hostname exists but not under this account
        // - abuse: Hostname blocked for abuse
        
        let parts: Vec<&str> = body.split_whitespace().collect();
        let status = parts.first().ok_or("Empty response from server")?;

        match *status {
            "good" => {
                log::info!("DNS record for {} successfully updated to {}", hostname, ip);
                Ok(())
            }
            "nochg" => {
                log::info!("DNS record for {} already set to {} (no change)", hostname, ip);
                Ok(())
            }
            "badauth" => Err("Bad authorization (username or password)".into()),
            "notfqdn" => Err("Not a fully-qualified domain name".into()),
            "nohost" => Err("Hostname doesn't exist".into()),
            "!yours" => Err("Hostname exists but not under this account".into()),
            "abuse" => Err("Hostname blocked for abuse".into()),
            "!donator" => Err("Feature requires donator account".into()),
            "!active" => Err("Hostname not activated".into()),
            "dnserr" => Err("DNS error on server".into()),
            _ => Err(format!("Unknown response: {}", body).into()),
        }
    }

    /// Ensures the client has both a username and a password configured.
    ///
    /// Returns `Ok(())` if both `username` and `password` are non-empty, `Err` with a descriptive message otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = crate::clients::dyndns2::DynDns2Client {
    ///     server: "https://members.dyndns.org".into(),
    ///     username: "user".into(),
    ///     password: "pass".into(),
    ///     script: "/nic/update".into(),
    /// };
    /// assert!(client.validate_config().is_ok());
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.username.is_empty() {
            return Err("username is required for DynDNS2".into());
        }
        if self.password.is_empty() {
            return Err("password is required for DynDNS2".into());
        }
        Ok(())
    }

    /// Provider identifier for this client.
    ///
    /// # Returns
    ///
    /// The provider name string: `"DynDNS2"`.
    ///
    /// # Examples
    ///
    /// ```
    /// let name = "DynDNS2";
    /// assert_eq!(name, "DynDNS2");
    /// ```
    fn provider_name(&self) -> &str {
        "DynDNS2"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> Config {
        Config {
            protocol: Some("dyndns2".to_string()),
            login: Some("testuser".to_string()),
            password: Some("testpass".to_string()),
            host: Some("ddns.example.com".to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn test_dyndns2_client_creation() {
        let config = create_test_config();
        let client = DynDns2Client::new(&config);
        
        assert!(client.is_ok());
        let client = client.unwrap();
        assert_eq!(client.username, "testuser");
        assert_eq!(client.password, "testpass");
        assert_eq!(client.server, "https://members.dyndns.org");
        assert_eq!(client.script, "/nic/update");
    }

    #[test]
    fn test_dyndns2_client_custom_server() {
        let config = Config {
            protocol: Some("dyndns2".to_string()),
            login: Some("testuser".to_string()),
            password: Some("testpass".to_string()),
            server: Some("https://custom.dyndns.com".to_string()),
            ..Default::default()
        };
        
        let client = DynDns2Client::new(&config);
        assert!(client.is_ok());
        assert_eq!(client.unwrap().server, "https://custom.dyndns.com");
    }

    #[test]
    fn test_dyndns2_missing_username() {
        let config = Config {
            protocol: Some("dyndns2".to_string()),
            login: None,
            password: Some("testpass".to_string()),
            ..Default::default()
        };
        
        let result = DynDns2Client::new(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("username is required"));
    }

    #[test]
    fn test_dyndns2_missing_password() {
        let config = Config {
            protocol: Some("dyndns2".to_string()),
            login: Some("testuser".to_string()),
            password: None,
            ..Default::default()
        };
        
        let result = DynDns2Client::new(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("password is required"));
    }

    #[test]
    fn test_dyndns2_validate_config_success() {
        let config = create_test_config();
        let client = DynDns2Client::new(&config).unwrap();
        
        let result = client.validate_config();
        assert!(result.is_ok());
    }

    #[test]
    fn test_dyndns2_validate_config_empty_fields() {
        let client = DynDns2Client {
            server: "https://test.com".to_string(),
            username: String::new(),
            password: String::new(),
            script: "/update".to_string(),
        };
        
        let result = client.validate_config();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("username is required"));
    }

    #[test]
    fn test_dyndns2_provider_name() {
        let config = create_test_config();
        let client = DynDns2Client::new(&config).unwrap();
        
        assert_eq!(client.provider_name(), "DynDNS2");
    }

    #[test]
    fn test_dyndns2_url_construction() {
        let config = create_test_config();
        let client = DynDns2Client::new(&config).unwrap();
        
        let hostname = "ddns.example.com";
        let ip = "203.0.113.1";
        let expected_url = format!(
            "{}{}?hostname={}&myip={}",
            client.server, client.script, hostname, ip
        );
        
        assert_eq!(expected_url, "https://members.dyndns.org/nic/update?hostname=ddns.example.com&myip=203.0.113.1");
    }

    #[test]
    fn test_dyndns2_auth_header() {
        let config = create_test_config();
        let client = DynDns2Client::new(&config).unwrap();
        
        // Verify that credentials are properly formatted for Basic auth
        let auth_string = format!("{}:{}", client.username, client.password);
        let encoded = general_purpose::STANDARD.encode(&auth_string);
        assert_eq!(encoded, general_purpose::STANDARD.encode("testuser:testpass"));
    }
}