use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

/// nsupdate DNS client
/// Uses RFC 2136 Dynamic DNS Update protocol
/// Note: This is a simplified implementation - full nsupdate would require TSIG/GSS-TSIG
pub struct NsupdateClient {
    server: String,
    username: String,
    password: String,
}

impl NsupdateClient {
    /// Creates a new `NsupdateClient` from the provided `Config`, extracting server, username, and password.
    ///
    /// Returns an error if `config.login` (zone/key name) or `config.password` (TSIG key) is missing.
    /// If `config.server` is not set, the server defaults to `"localhost"`.
    ///
    /// # Examples
    ///
    /// ```
    /// let cfg = Config {
    ///     login: Some("example-zone".to_string()),
    ///     password: Some("secret-tsig-key".to_string()),
    ///     server: Some("dns.example.com".to_string()),
    ///     ..Default::default()
    /// };
    /// let client = NsupdateClient::new(&cfg).unwrap();
    /// assert_eq!(client.provider_name(), "nsupdate");
    /// ```
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let username = config.login.as_ref()
            .ok_or("username (zone/key name) is required for nsupdate")?
            .clone();
        let password = config.password.as_ref()
            .ok_or("password (TSIG key) is required for nsupdate")?
            .clone();
        
        let server = config.server.clone()
            .unwrap_or_else(|| "localhost".to_string());

        Ok(NsupdateClient {
            server,
            username,
            password,
        })
    }
}

impl DnsClient for NsupdateClient {
    /// Attempt to update a DNS record for `hostname` to the given `ip` using nsupdate (placeholder).
    ///
    /// This implementation is a stub and does not perform RFC 2136 DNS updates; it logs the attempted operation
    /// and returns an error indicating that proper DNS protocol/library support is required.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::IpAddr;
    /// use crate::clients::nsupdate::NsupdateClient;
    /// use crate::config::Config;
    ///
    /// let cfg = Config { login: Some("example.com".into()), password: Some("tsig-key".into()), server: Some("127.0.0.1".into()) };
    /// let client = NsupdateClient::new(&cfg).unwrap();
    /// let ip: IpAddr = "192.0.2.1".parse().unwrap();
    /// let res = client.update_record("host.example.com.", ip);
    /// assert!(res.is_err());
    /// ```
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, `Err` with an explanatory error if the operation is not implemented because RFC 2136 support is missing.
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        // Note: This is a placeholder for nsupdate functionality
        // A full implementation would require DNS protocol handling (RFC 2136)
        // For now, we'll return an error indicating this needs proper DNS library support
        
        log::info!("nsupdate: {} -> {} (via {})", hostname, ip, self.server);
        
        Err(format!(
            "nsupdate requires DNS protocol library support (RFC 2136). \
             Server: {}, Zone: {}, Record: {} -> {}. \
             Consider using a dedicated nsupdate tool or DNS library.",
            self.server, self.username, hostname, ip
        ).into())
    }

    /// Validates that the client has the required nsupdate credentials.
    ///
    /// Returns `Ok(())` if both the zone/key name (`username`) and TSIG key (`password`) are present,
    /// `Err` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::clients::nsupdate::NsupdateClient;
    /// use crate::config::Config;
    ///
    /// let cfg = Config { login: Some("example-zone".into()), password: Some("secret-tsig".into()), server: None };
    /// let client = NsupdateClient::new(&cfg).unwrap();
    /// assert!(client.validate_config().is_ok());
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.username.is_empty() {
            return Err("username (zone/key name) is required for nsupdate".into());
        }
        if self.password.is_empty() {
            return Err("password (TSIG key) is required for nsupdate".into());
        }
        Ok(())
    }

    /// Provider name for this DNS client.
    ///
    /// Returns the provider identifier `"nsupdate"`.
    fn provider_name(&self) -> &str {
        "nsupdate"
    }
}