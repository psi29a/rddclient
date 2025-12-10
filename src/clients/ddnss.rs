use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

/// DDNSS.de DNS client
/// Uses simple token-based GET protocol
pub struct DdnssClient {
    server: String,
    token: String,
}

impl DdnssClient {
    /// Create a DdnssClient from a Config, using the config's password as the API token.
    ///
    /// If `config.password` is missing an error is returned. If `config.server` is not provided,
    /// the server defaults to "https://www.ddnss.de".
    ///
    /// # Examples
    ///
    /// ```
    /// let cfg = Config { server: None, password: Some("secret-token".to_string()), ..Default::default() };
    /// let client = DdnssClient::new(&cfg).expect("should create client");
    /// assert_eq!(client.provider_name(), "DDNSS");
    /// ```
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let token = config.password.as_ref()
            .ok_or("password (token) is required for DDNSS")?
            .clone();
        
        let server = config.server.as_ref()
            .cloned()
            .unwrap_or_else(|| "https://www.ddnss.de".to_string());

        Ok(DdnssClient {
            server,
            token,
        })
    }
}

impl DdnssClient {
    /// Updates the DNS record for `hostname` at DDNSS to the specified `ip`.
    ///
    /// On success, the provider confirmed the update (response contains `"Updated"` or `"good"`).
    ///
    /// Errors:
    /// - Returns an error if the HTTP status code is not 200.
    /// - Returns an error with message `Authentication failed - check token` if the provider response contains `"badauth"` or `"Authentication"`.
    /// - Returns an error with message `Hostname does not exist` if the provider response contains `"nohost"`.
    /// - Returns an error containing the provider body for any other unexpected response.
    /// - Network or request failures are returned as underlying I/O errors.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::IpAddr;
    ///
    /// // Construct a client (fields shown for example; use Config::new(...) in real code).
    /// let client = crate::clients::ddnss::DdnssClient {
    ///     server: "https://www.ddnss.de".to_string(),
    ///     token: "example-token".to_string(),
    /// };
    ///
    /// // Attempt to update; result indicates whether the provider accepted the update.
    /// let res = client.update_record("my.host.example", "1.2.3.4".parse::<IpAddr>().unwrap());
    /// // The call may fail in examples/environments without network or with invalid token.
    /// let _ = res;
    /// ```
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let url = format!(
            "{}/upd.php?key={}&host={}&ip={}",
            self.server, self.token, hostname, ip
        );

        log::info!("Updating {} with DDNSS", hostname);

        let response = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .send()?;

        let status_code = response.status_code;
        let body = response.as_str()?.trim();

        log::debug!("Response status: {}, body: {}", status_code, body);

        if status_code != 200 {
            return Err(format!("HTTP error: {}", status_code).into());
        }

        // Parse DDNSS response
        if body.contains("Updated") || body.contains("good") {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else if body.contains("badauth") || body.contains("Authentication") {
            Err("Authentication failed - check token".into())
        } else if body.contains("nohost") {
            Err("Hostname does not exist".into())
        } else {
            Err(format!("Unexpected response: {}", body).into())
        }
    }

    /// Ensure the client has a non-empty API token required by DDNSS.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the client's token is non-empty, `Err` with a descriptive message otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = DdnssClient { server: "https://www.ddnss.de".into(), token: "secret".into() };
    /// assert!(client.validate_config().is_ok());
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.token.is_empty() {
            return Err("password (token) is required for DDNSS".into());
        }
        Ok(())
    }

    /// Provider identifier for this client.
    ///
    /// # Returns
    ///
    /// `"DDNSS"` — the provider name.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = crate::clients::ddnss::DdnssClient { server: String::new(), token: String::new() };
    /// assert_eq!(client.provider_name(), "DDNSS");
    /// ```
    fn provider_name(&self) -> &str {
        "DDNSS"
    }
}

impl DnsClient for DdnssClient {
    /// Update the DNS record for `hostname` to the specified `ip` using the DDNSS provider.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::IpAddr;
    /// use crate::clients::ddnss::DdnssClient;
    ///
    /// let client = DdnssClient { server: String::from("https://www.ddnss.de"), token: String::from("secret") };
    /// let ip: IpAddr = "198.51.100.42".parse().unwrap();
    /// let _ = client.update_record("example.ddnss.de", ip);
    /// ```
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        DdnssClient::update_record(self, hostname, ip)
    }

    /// Validates that the DDNSS client has a usable configuration.
    ///
    /// Returns `Ok(())` if the client's token (password) is present, or an `Err` describing the validation failure otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// // Construct a Config with a password (token) and optional server, then validate.
    /// let cfg = Config { server: None, username: None, password: Some("secret".into()), ..Default::default() };
    /// let client = DdnssClient::new(&cfg).unwrap();
    /// client.validate_config().unwrap();
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        DdnssClient::validate_config(self)
    }

    /// The DNS provider name for this client.
    ///
    /// Returns the provider name "DDNSS".
    ///
    /// # Examples
    ///
    /// ```
    /// let client = DdnssClient { server: "https://www.ddnss.de".into(), token: "token".into() };
    /// assert_eq!(client.provider_name(), "DDNSS");
    /// ```
    fn provider_name(&self) -> &str {
        DdnssClient::provider_name(self)
    }
}