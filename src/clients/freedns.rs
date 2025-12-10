use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

/// Freedns (afraid.org) client - https://freedns.afraid.org/
pub struct FreednsClient {
    token: String,
    server: String,
}

impl FreednsClient {
    /// Create a FreednsClient from the given configuration.
    ///
    /// The configuration must provide a token in `password` (or `api_token`); if missing, this returns an error with message `"token (password or api_token) is required for Freedns"`.
    /// If `server` is not provided, the Freedns dynamic update endpoint
    /// "https://freedns.afraid.org/dynamic" is used.
    ///
    /// # Examples
    ///
    /// ```
    /// // Minimal local Config used only for the example
    /// struct Config { password: Option<String>, server: Option<String> }
    ///
    /// let cfg = Config {
    ///     password: Some("my-token".to_string()),
    ///     server: None,
    /// };
    ///
    /// let client = FreednsClient::new(&cfg).unwrap();
    /// assert_eq!(client.server, "https://freedns.afraid.org/dynamic");
    /// ```
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let token = config.password.as_ref()
            .or(config.password.as_ref())
            .ok_or("token (password or api_token) is required for Freedns")?
            .clone();
        
        let server = config.server.as_ref()
            .cloned()
            .unwrap_or_else(|| "https://freedns.afraid.org/dynamic".to_string());

        Ok(FreednsClient { token, server })
    }
}

impl DnsClient for FreednsClient {
    /// Update a DNS record on Freedns (freedns.afraid.org) for the given hostname to the specified IP address.
    ///
    /// The method sends an update request to the Freedns dynamic-update endpoint and returns `Ok(())` when the service
    /// reports the record was updated or "has not changed". If the service returns an error message or an unexpected
    /// response, an `Err` carrying a descriptive message is returned.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::IpAddr;
    /// let ip: IpAddr = "1.2.3.4".parse().unwrap();
    /// // `client` is a configured FreednsClient
    /// // client.update_record("example.com", ip).unwrap();
    /// ```
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        // Freedns uses a unique token per host
        let url = format!("{}/update.php?{}&address={}", self.server, self.token, ip);

        log::info!("Updating {} with Freedns", hostname);

        let response = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .send()?;

        let body = response.as_str()?;

        if body.contains("Updated") || body.contains("has not changed") {
            log::info!("DNS record for {} successfully updated to {}", hostname, ip);
            Ok(())
        } else if body.contains("ERROR") {
            Err(format!("Freedns error: {}", body).into())
        } else {
            Err(format!("Unexpected response: {}", body).into())
        }
    }

    /// Ensures the client has a non-empty API token required by the Freedns provider.
    ///
    /// # Errors
    ///
    /// Returns an `Err` containing a descriptive message if the client's `token` is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = FreednsClient { token: "api_token".into(), server: "https://freedns.afraid.org/dynamic".into() };
    /// assert!(client.validate_config().is_ok());
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.token.is_empty() {
            return Err("token is required for Freedns".into());
        }
        Ok(())
    }

    /// Provider identifier for the Freedns client.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = FreednsClient { token: String::from("t"), server: String::from("s") };
    /// assert_eq!(client.provider_name(), "Freedns");
    /// ```
    fn provider_name(&self) -> &str {
        "Freedns"
    }
}