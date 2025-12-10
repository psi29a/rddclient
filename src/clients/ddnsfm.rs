use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

/// DDNS.FM DNS client
/// Uses DDNS.FM REST API
pub struct DdnsfmClient {
    server: String,
    token: String,
}

impl DdnsfmClient {
    /// Create a DDNS.FM client from the provided configuration.
    ///
    /// The configuration must include the API token in `config.password`. If `config.server` is
    /// omitted, the default server "https://api.ddns.fm" is used.
    ///
    /// # Parameters
    ///
    /// - `config`: configuration containing the `password` (token) and an optional `server` URL.
    ///
    /// # Returns
    ///
    /// `Ok(DdnsfmClient)` on success, or an `Err` if `config.password` is missing (error message:
    /// "password (token) is required for DDNS.FM").
    ///
    /// # Examples
    ///
    /// ```
    /// let cfg = Config {
    ///     password: Some("my-secret-token".to_string()),
    ///     server: None,
    ///     ..Default::default()
    /// };
    /// let client = DdnsfmClient::new(&cfg).unwrap();
    /// assert_eq!(client.provider_name(), "DDNS.FM");
    /// ```
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let token = config.password.as_ref()
            .ok_or("password (token) is required for DDNS.FM")?
            .clone();
        
        let server = config.server.clone()
            .unwrap_or_else(|| "https://api.ddns.fm".to_string());

        Ok(DdnsfmClient {
            server,
            token,
        })
    }
}

impl DnsClient for DdnsfmClient {
    /// Update the DNS record for `hostname` to the provided `ip` using the DDNS.FM API.
    ///
    /// Sends a GET request to the configured DDNS.FM server's `/update` endpoint with the
    /// client's token, the hostname, and the IP address. Returns `Ok(())` when the API
    /// response indicates success; returns `Err` when the HTTP status is not 200 or when
    /// the API response reports an error or an unexpected result.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::IpAddr;
    /// let client = DdnsfmClient { server: "https://api.ddns.fm".into(), token: "secret".into() };
    /// let ip: IpAddr = "1.2.3.4".parse().unwrap();
    /// let result = client.update_record("example.com", ip);
    /// assert!(result.is_ok());
    /// ```
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        log::info!("Updating {} with DDNS.FM", hostname);

        // DDNS.FM API endpoint
        let url = format!("{}/update", self.server);

        let response = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .with_param("token", &self.token)
            .with_param("hostname", hostname)
            .with_param("ip", ip.to_string())
            .send()?;

        let status_code = response.status_code;
        let body = response.as_str()?.trim();

        log::debug!("Response status: {}, body: {}", status_code, body);

        if status_code != 200 {
            return Err(format!("HTTP error: {}", status_code).into());
        }

        // Check for success indicators
        if body.contains("success") || body.contains("updated") || body == "OK" {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else if body.contains("error") || body.contains("fail") {
            Err(format!("DDNS.FM error: {}", body).into())
        } else {
            Err(format!("Unexpected response: {}", body).into())
        }
    }

    /// Validates that the client has a non-empty token required for DDNS.FM.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the client's token is not empty.
    ///
    /// # Errors
    ///
    /// Returns an `Err` with the message `"password (token) is required for DDNS.FM"` if the token is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// let good = DdnsfmClient { server: "https://api.ddns.fm".into(), token: "secret".into() };
    /// assert!(good.validate_config().is_ok());
    ///
    /// let bad = DdnsfmClient { server: "https://api.ddns.fm".into(), token: "".into() };
    /// assert!(bad.validate_config().is_err());
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.token.is_empty() {
            return Err("password (token) is required for DDNS.FM".into());
        }
        Ok(())
    }

    /// Provides the DNS provider name for this client.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = DdnsfmClient { server: "https://api.ddns.fm".into(), token: "token".into() };
    /// assert_eq!(client.provider_name(), "DDNS.FM");
    /// ```
    fn provider_name(&self) -> &str {
        "DDNS.FM"
    }
}