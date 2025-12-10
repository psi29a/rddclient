use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

/// Afraid.org DNS client (v2 API)
/// Uses Afraid.org's update API with token
pub struct AfraidClient {
    server: String,
    token: String,
}

impl AfraidClient {
    /// Creates an AfraidClient from configuration by extracting the update token and server URL.
    ///
    /// The function requires `config.password` to contain the Afraid.org update token; if `config.server` is
    /// present it will be used as the API server URL, otherwise `https://freedns.afraid.org` is used.
    ///
    /// # Errors
    ///
    /// Returns an error if `config.password` is `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// let config = Config {
    ///     password: Some("update-token".to_string()),
    ///     server: None,
    ///     ..Default::default()
    /// };
    /// let client = AfraidClient::new(&config).unwrap();
    /// assert_eq!(client.provider_name(), "Afraid.org");
    /// ```
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let token = config.password.as_ref()
            .ok_or("api_token (update token) is required for Afraid.org")?
            .clone();
        
        let server = config.server.clone()
            .unwrap_or_else(|| "https://freedns.afraid.org".to_string());

        Ok(AfraidClient {
            server,
            token,
        })
    }
}

impl DnsClient for AfraidClient {
    /// Update the DNS record for `hostname` to the given IP using the Afraid.org dynamic DNS API.
    ///
    /// Sends a GET request to the Afraid.org API with the client's configured token and the provided
    /// hostname and IP, and treats responses containing "Updated" or "has not changed" as success.
    /// Returns an error if the HTTP status is not 200, the provider returns an `ERROR` message, or
    /// the response is otherwise unexpected.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success; `Err` containing a description of the HTTP or provider error otherwise.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Example usage (network call; ignored in doctest):
    /// let client = AfraidClient::new(&config).unwrap();
    /// let ip: std::net::IpAddr = "203.0.113.42".parse().unwrap();
    /// client.update_record("example.com", ip).expect("update failed");
    /// ```
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        log::info!("Updating {} with Afraid.org", hostname);

        // Afraid.org API endpoint with token
        let url = format!("{}/api/?action=getdyndns&sha={}", self.server, self.token);

        let response = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .with_param("hostname", hostname)
            .with_param("myip", ip.to_string())
            .send()?;

        let status_code = response.status_code;
        let body = response.as_str()?.trim();

        log::debug!("Response status: {}, body: {}", status_code, body);

        if status_code != 200 {
            return Err(format!("HTTP error: {}", status_code).into());
        }

        // Check for success indicators
        if body.contains("Updated") || body.contains("has not changed") {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else if body.contains("ERROR") {
            Err(format!("Afraid.org error: {}", body).into())
        } else {
            Err(format!("Unexpected response: {}", body).into())
        }
    }

    /// Validates that the client has a configured Afraid.org update token.
    ///
    /// Returns `Ok(())` if the client's update token is non-empty; returns `Err` with an explanatory message otherwise.
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.token.is_empty() {
            return Err("api_token (update token) is required for Afraid.org".into());
        }
        Ok(())
    }

    /// DNS provider identifier for this client.
    ///
    /// # Returns
    ///
    /// `&str` containing the provider name "Afraid.org".
    fn provider_name(&self) -> &str {
        "Afraid.org"
    }
}