use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

pub struct NjallaClient {
    api_key: String,
    server: String,
}

impl NjallaClient {
    /// Creates a NjallaClient from the given configuration.
    ///
    /// Uses `config.password` as the API key and `config.server` as the server URL, defaulting to "https://njal.la" when `config.server` is not set.
    ///
    /// # Errors
    /// Returns an error if `config.password` is `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// let cfg = Config {
    ///     password: Some("api_key".to_string()),
    ///     server: None,
    ///     // other fields...
    /// };
    /// let client = NjallaClient::new(&cfg).unwrap();
    /// assert_eq!(client.provider_name(), "Njalla");
    /// ```
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let api_key = config.password.as_ref()
            .ok_or("Njalla requires API key (use password)")?
            .clone();
        let server = config.server.clone()
            .unwrap_or_else(|| "https://njal.la".to_string());

        Ok(Self {
            api_key,
            server,
        })
    }
}

impl DnsClient for NjallaClient {
    /// Update the DNS record for `hostname` to the specified `ip` using the Njalla API.
    ///
    /// On success the function completes without a value; on failure it returns an error describing
    /// either the HTTP status or the provider response body.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Assuming `client` is a configured NjallaClient:
    /// let _ = client.update_record("example.com", "203.0.113.1".parse().unwrap())?;
    /// ```
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let url = format!("{}/update?h={}&k={}&a={}", 
            self.server, hostname, self.api_key, ip);
        
        log::info!("Updating {} to {}", hostname, ip);
        
        let response = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .send()?;

        // Njalla returns status 200 on success
        if response.status_code == 200 {
            let body = response.as_str()?;
            // Empty response or contains success indicators
            if body.is_empty() || !body.to_lowercase().contains("error") {
                log::info!("Successfully updated {} to {}", hostname, ip);
                return Ok(());
            }
            return Err(format!("Update failed: {}", body).into());
        }

        Err(format!("HTTP error: {}", response.status_code).into())
    }

    /// Validates that the client has a configured API key.
    ///
    /// Returns `Ok(())` if the API key is non-empty, otherwise returns an error describing the missing key.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = NjallaClient { api_key: "key".into(), server: "https://njal.la".into() };
    /// assert!(client.validate_config().is_ok());
    ///
    /// let bad = NjallaClient { api_key: "".into(), server: "https://njal.la".into() };
    /// assert!(bad.validate_config().is_err());
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.api_key.is_empty() {
            return Err("Njalla API key cannot be empty".into());
        }
        Ok(())
    }

    /// DNS provider display name.
    ///
    /// # Returns
    ///
    /// The provider's name as a string slice.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = NjallaClient { api_key: String::new(), server: "https://njal.la".into() };
    /// assert_eq!(client.provider_name(), "Njalla");
    /// ```
    fn provider_name(&self) -> &str {
        "Njalla"
    }
}