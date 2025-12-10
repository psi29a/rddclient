use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

pub struct DirectnicClient {
    urlv4: Option<String>,
    urlv6: Option<String>,
}

impl DirectnicClient {
    /// Create a DirectnicClient from configuration.
    ///
    /// The `server` field of `config` is used as the IPv4 update URL (`urlv4`) and the
    /// `password` field of `config` is used as the IPv6 update URL (`urlv6`). At least
    /// one of these must be present; otherwise an error is returned.
    ///
    /// # Errors
    ///
    /// Returns an error if both `server` and `password` are `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// // Construct a `Config` with at least one of `server` or `password` set,
    /// // then create the client.
    /// let cfg = Config { server: Some("https://ipv4.example/update".into()), password: None, ..Default::default() };
    /// let client = DirectnicClient::new(&cfg).expect("should construct DirectnicClient");
    /// ```
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        // For Directnic, we use server for urlv4 and password for urlv6
        let urlv4 = config.server.clone();
        let urlv6 = config.password.clone();

        // At least one URL must be provided
        if urlv4.is_none() && urlv6.is_none() {
            return Err("At least one of urlv4 (server) or urlv6 (password) is required for Directnic".into());
        }

        Ok(DirectnicClient {
            urlv4,
            urlv6,
        })
    }
}

impl DnsClient for DirectnicClient {
    /// Update the Directnic DNS record for `hostname` to the provided `ip`.
    ///
    /// Selects the configured IPv4 or IPv6 update URL based on the IP address type and performs the HTTP request to apply the change.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the DNS update succeeded; `Err` containing a descriptive message if the client is not configured for the IP version, the HTTP request fails, or the provider responds with a non-200 status.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::IpAddr;
    ///
    /// // Construct a client with an IPv4 update URL for the example.
    /// let client = DirectnicClient { urlv4: Some("http://example.com/update".into()), urlv6: None };
    /// let ip: IpAddr = "1.2.3.4".parse().unwrap();
    /// let result = client.update_record("host.example.com", ip);
    /// assert!(result.is_ok());
    /// ```
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        log::info!("Updating Directnic record for {} to {}", hostname, ip);

        // Select the appropriate URL based on IP address type
        let url = match ip {
            IpAddr::V4(_) => {
                self.urlv4.as_ref().ok_or("urlv4 not configured for IPv4 address")?
            }
            IpAddr::V6(_) => {
                self.urlv6.as_ref().ok_or("urlv6 not configured for IPv6 address")?
            }
        };

        // Directnic uses a simple GET request to the provided URL
        let response = minreq::get(url)
            .with_header("User-Agent", crate::USER_AGENT)
            .send()?;

        if response.status_code == 200 {
            log::info!("Successfully updated DNS record for {} to {}", hostname, ip);
            Ok(())
        } else {
            let body = response.as_str().unwrap_or("No response body");
            Err(format!(
                "Directnic API error: HTTP {} - {}",
                response.status_code, body
            )
            .into())
        }
    }

    /// Validate that the Directnic client configuration contains at least one URL and that any provided URLs use an HTTP or HTTPS scheme.
    ///
    /// Returns `Ok(())` if at least one of `urlv4` or `urlv6` is set and each provided URL starts with `http://` or `https://`.
    /// Returns `Err` if neither URL is configured or if a provided URL does not start with `http://` or `https://`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::error::Error;
    ///
    /// // Construct a DirectnicClient directly for the example.
    /// let client = crate::clients::directnic::DirectnicClient {
    ///     urlv4: Some("https://example.com/update".into()),
    ///     urlv6: None,
    /// };
    ///
    /// assert!(client.validate_config().is_ok());
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.urlv4.is_none() && self.urlv6.is_none() {
            return Err("At least one of urlv4 or urlv6 must be configured for Directnic".into());
        }
        
        // Validate URLs if provided
        if let Some(url) = &self.urlv4 {
            if !url.starts_with("http://") && !url.starts_with("https://") {
                return Err("urlv4 must start with http:// or https://".into());
            }
        }
        if let Some(url) = &self.urlv6 {
            if !url.starts_with("http://") && !url.starts_with("https://") {
                return Err("urlv6 must start with http:// or https://".into());
            }
        }
        
        Ok(())
    }

    /// Return the DNS provider name for this client.
    ///
    /// # Returns
    ///
    /// The provider name string `"Directnic"`.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = DirectnicClient { urlv4: None, urlv6: None };
    /// assert_eq!(client.provider_name(), "Directnic");
    /// ```
    fn provider_name(&self) -> &'static str {
        "Directnic"
    }
}