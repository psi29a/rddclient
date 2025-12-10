use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

/// Freemyip DNS client
/// Uses simple token-based GET protocol
pub struct FreemyipClient {
    server: String,
    token: String,
}

impl FreemyipClient {
    /// Create a `FreemyipClient` from the provided configuration.
    ///
    /// The function extracts the required token from `config.password` and uses
    /// `config.server` if present; otherwise it defaults the server to
    /// "https://freemyip.com".
    ///
    /// # Parameters
    ///
    /// - `config`: Configuration containing `password` (token) and optional `server`.
    ///
    /// # Returns
    ///
    /// `Ok(FreemyipClient)` initialized with the resolved server and token, or an
    /// `Err` if the configuration does not contain the required token.
    ///
    /// # Examples
    ///
    /// ```
    /// let config = Config {
    ///     password: Some("my-token".to_string()),
    ///     server: Some("https://freemyip.com".to_string()),
    ///     ..Default::default()
    /// };
    /// let client = FreemyipClient::new(&config).unwrap();
    /// assert_eq!(client.provider_name(), "Freemyip");
    /// ```
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let token = config.password.as_ref()
            .ok_or("password (token) is required for Freemyip")?
            .clone();
        
        let server = config.server.as_ref()
            .cloned()
            .unwrap_or_else(|| "https://freemyip.com".to_string());

        Ok(FreemyipClient {
            server,
            token,
        })
    }
}

impl DnsClient for FreemyipClient {
    /// Update the DNS record for `hostname` at the Freemyip service.
    ///
    /// Performs an HTTP GET against the configured Freemyip server using the client's token and the given hostname.
    /// On success, logs and returns `Ok(())`.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the provider acknowledged the update (`SUCCESS`, `UPDATED`, or `OK` in the response);
    /// `Err` if the HTTP status is not 200, the response contains `ERROR`, or the response is otherwise unexpected — the error contains the HTTP status or provider body.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// // Construct a FreemyipClient (example omitted) and call update_record:
    /// // let client = FreemyipClient::new(&config).unwrap();
    /// // client.update_record("example.com", "1.2.3.4".parse().unwrap()).unwrap();
    /// ```
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let url = format!(
            "{}/update?token={}&domain={}",
            self.server, self.token, hostname
        );

        log::info!("Updating {} with Freemyip", hostname);

        let response = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .send()?;

        let status_code = response.status_code;
        let body = response.as_str()?.trim();

        log::debug!("Response status: {}, body: {}", status_code, body);

        if status_code != 200 {
            return Err(format!("HTTP error: {}", status_code).into());
        }

        // Parse Freemyip response
        if body.contains("SUCCESS") || body.contains("UPDATED") || body == "OK" {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else if body.contains("ERROR") {
            Err(format!("Freemyip error: {}", body).into())
        } else {
            Err(format!("Unexpected response: {}", body).into())
        }
    }

    /// Validates that the client has a non-empty token.
    ///
    /// Returns `Err` if the client's token is an empty string; otherwise returns `Ok(())`.
    ///
    /// # Examples
    ///
    /// ```
    /// let good = FreemyipClient { server: "https://freemyip.com".into(), token: "token".into() };
    /// assert!(good.validate_config().is_ok());
    ///
    /// let bad = FreemyipClient { server: "https://freemyip.com".into(), token: "".into() };
    /// assert!(bad.validate_config().is_err());
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.token.is_empty() {
            return Err("password (token) is required for Freemyip".into());
        }
        Ok(())
    }

    /// Provider name for this DNS client.
    ///
    /// # Returns
    ///
    /// The provider name, `"Freemyip"`.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = crate::clients::freemyip::FreemyipClient { server: String::new(), token: String::new() };
    /// assert_eq!(client.provider_name(), "Freemyip");
    /// ```
    fn provider_name(&self) -> &str {
        "Freemyip"
    }
}