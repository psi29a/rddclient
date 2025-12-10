use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

/// Dinahosting DNS client
/// Uses Dinahosting's REST API with basic authentication
pub struct DinahostingClient {
    server: String,
    username: String,
    password: String,
}

impl DinahostingClient {
    /// Creates a new DinahostingClient from configuration, requiring a username and password and defaulting the server to "https://dinahosting.com" when not provided.
    ///
    /// Returns an error if the configuration does not contain a username or password.
    ///
    /// # Examples
    ///
    /// ```
    /// // Construct a Config with required fields (type shown for clarity; use the project's Config)
    /// let config = Config {
    ///     login: Some("user@example.com".to_string()),
    ///     password: Some("s3cret".to_string()),
    ///     server: None,
    ///     ..Default::default()
    /// };
    ///
    /// let client = DinahostingClient::new(&config).unwrap();
    /// assert_eq!(client.server, "https://dinahosting.com");
    /// assert_eq!(client.username, "user@example.com");
    /// ```
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let username = config.login.as_ref()
            .ok_or("username is required for Dinahosting")?
            .clone();
        let password = config.password.as_ref()
            .ok_or("password is required for Dinahosting")?
            .clone();
        
        let server = config.server.as_ref()
            .cloned()
            .unwrap_or_else(|| "https://dinahosting.com".to_string());

        Ok(DinahostingClient {
            server,
            username,
            password,
        })
    }

    /// Derives the domain from a hostname by removing its first dot-separated label.
    ///
    /// Returns the domain portion of `hostname` (e.g., `"ddns.example.com"` -> `"example.com"`).
    ///
    /// # Examples
    ///
    /// ```
    /// let client = DinahostingClient { server: String::new(), username: String::new(), password: String::new() };
    /// assert_eq!(client.get_domain_from_hostname("ddns.example.com"), "example.com");
    /// ```
    fn get_domain_from_hostname(&self, hostname: &str) -> String {
        // Extract domain from hostname (e.g., "ddns.example.com" -> "example.com")
        hostname.split('.').skip(1).collect::<Vec<_>>().join(".")
    }
}

impl DnsClient for DinahostingClient {
    /// Update the DNS record for `hostname` to the provided `ip` using Dinahosting's DynDNS API.
    ///
    /// Sends a GET request to Dinahosting's API to set an A (IPv4) or AAAA (IPv6) record for the given hostname.
    /// On success (API response indicates success), the function returns `Ok(())`. On failure it returns an `Err` describing:
    /// - a non-200 HTTP status code,
    /// - an authentication failure reported by the provider,
    /// - a provider-specific error message, or
    /// - an unexpected response body.
    ///
    /// # Errors
    ///
    /// Returns an `Err` when the HTTP request fails, the status code is not 200, or the API response indicates an error (including authentication failures).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::net::IpAddr;
    /// # use your_crate::clients::dinahosting::DinahostingClient;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = DinahostingClient {
    ///     server: "https://dinahosting.com".into(),
    ///     username: "user".into(),
    ///     password: "pass".into(),
    /// };
    /// let ip: IpAddr = "1.2.3.4".parse()?;
    /// client.update_record("ddns.example.com", ip)?;
    /// # Ok(())
    /// # }
    /// ```
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let domain = self.get_domain_from_hostname(hostname);
        let record_type = match ip {
            IpAddr::V4(_) => "A",
            IpAddr::V6(_) => "AAAA",
        };

        let url = format!(
            "{}/special/api.php?AUTH_USER={}&AUTH_PWD={}&command=Domain_Zone_UpdateDynDNS&domain={}&zone={}&type={}&ip={}",
            self.server, self.username, self.password, domain, hostname, record_type, ip
        );

        log::info!("Updating {} with Dinahosting", hostname);

        let response = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .send()?;

        let status_code = response.status_code;
        let body = response.as_str()?.trim();

        log::debug!("Response status: {}, body: {}", status_code, body);

        if status_code != 200 {
            return Err(format!("HTTP error: {}", status_code).into());
        }

        // Parse response
        if body.contains("responseStatus=ok") || body.contains("success") {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else if body.contains("responseStatus=error") {
            if body.contains("authentication") {
                Err("Authentication failed - check username and password".into())
            } else {
                Err(format!("Dinahosting error: {}", body).into())
            }
        } else {
            Err(format!("Unexpected response: {}", body).into())
        }
    }

    /// Ensures the client has both a username and a password configured.
    ///
    /// # Errors
    ///
    /// Returns an error if the username is empty or if the password is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = DinahostingClient {
    ///     server: "https://dinahosting.com".into(),
    ///     username: "user".into(),
    ///     password: "pass".into(),
    /// };
    /// assert!(client.validate_config().is_ok());
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.username.is_empty() {
            return Err("username is required for Dinahosting".into());
        }
        if self.password.is_empty() {
            return Err("password is required for Dinahosting".into());
        }
        Ok(())
    }

    /// Gets the DNS provider name for this client.
    ///
    /// Returns the provider name "Dinahosting".
    ///
    /// # Examples
    ///
    /// ```
    /// let client = DinahostingClient {
    ///     server: String::from("https://dinahosting.com"),
    ///     username: String::from("user"),
    ///     password: String::from("pass"),
    /// };
    /// assert_eq!(client.provider_name(), "Dinahosting");
    /// ```
    fn provider_name(&self) -> &str {
        "Dinahosting"
    }
}