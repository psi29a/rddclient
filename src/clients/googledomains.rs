use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;
use base64::{Engine as _, engine::general_purpose};

/// Google Domains DNS client
/// Uses DynDNS2 protocol with basic authentication
pub struct GoogleDomainsClient {
    server: String,
    username: String,
    password: String,
}

impl GoogleDomainsClient {
    /// Creates a `GoogleDomainsClient` from a `Config`.
    ///
    /// The provided `Config` must include `login` (username) and `password`; if `server` is omitted the client will use
    /// "https://domains.google.com".
    ///
    /// # Errors
    ///
    /// Returns an error if `login` or `password` are not present in `config`.
    ///
    /// # Examples
    ///
    /// ```
    /// let config = Config {
    ///     login: Some("user@example.com".into()),
    ///     password: Some("s3cret".into()),
    ///     server: None,
    ///     ..Default::default()
    /// };
    /// let client = GoogleDomainsClient::new(&config).unwrap();
    /// assert_eq!(client.provider_name(), "Google Domains");
    /// ```
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let username = config.login.as_ref()
            .ok_or("username is required for Google Domains")?
            .clone();
        let password = config.password.as_ref()
            .ok_or("password is required for Google Domains")?
            .clone();
        
        let server = config.server.as_ref()
            .cloned()
            .unwrap_or_else(|| "https://domains.google.com".to_string());

        Ok(GoogleDomainsClient {
            server,
            username,
            password,
        })
    }
}

impl DnsClient for GoogleDomainsClient {
    /// Update a DNS A record for a hostname using Google Domains' DynDNS2-compatible API.
    ///
    /// Sends an update request for `hostname` to the configured Google Domains server and interprets
    /// DynDNS2-style responses to determine success or failure.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the provider accepted the update; `Err` with a descriptive message for HTTP errors,
    /// authentication failures, invalid hostname format, nonexistent hostnames, abuse blocks, provider
    /// server errors, or any unexpected provider response.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::IpAddr;
    ///
    /// let client = GoogleDomainsClient {
    ///     server: "https://domains.google.com".into(),
    ///     username: "user".into(),
    ///     password: "pass".into(),
    /// };
    ///
    /// let ip: IpAddr = "1.2.3.4".parse().unwrap();
    /// client.update_record("example.com", ip).unwrap();
    /// ```
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let url = format!(
            "{}/nic/update?hostname={}&myip={}",
            self.server, hostname, ip
        );

        log::info!("Updating {} with Google Domains", hostname);

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

        // Parse DynDNS2-style response
        if body.starts_with("good") || body.starts_with("nochg") {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else if body.starts_with("badauth") {
            Err("Authentication failed - check username and password".into())
        } else if body.starts_with("notfqdn") {
            Err("Invalid hostname format".into())
        } else if body.starts_with("nohost") {
            Err("Hostname does not exist".into())
        } else if body.starts_with("abuse") {
            Err("Account blocked for abuse".into())
        } else if body.starts_with("911") {
            Err("Server error - try again later".into())
        } else {
            Err(format!("Unexpected response: {}", body).into())
        }
    }

    /// Validates that the client has both a username and a password configured.
    ///
    /// Returns `Ok(())` if both `username` and `password` are non-empty, or an `Err` with a
    /// descriptive message when either field is missing.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = GoogleDomainsClient {
    ///     server: "https://domains.google.com".into(),
    ///     username: "user".into(),
    ///     password: "pass".into(),
    /// };
    /// assert!(client.validate_config().is_ok());
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.username.is_empty() {
            return Err("username is required for Google Domains".into());
        }
        if self.password.is_empty() {
            return Err("password is required for Google Domains".into());
        }
        Ok(())
    }

    /// Provider display name for this client.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = crate::clients::googledomains::GoogleDomainsClient {
    ///     server: String::from("https://domains.google.com"),
    ///     username: String::from("user"),
    ///     password: String::from("pass"),
    /// };
    /// assert_eq!(client.provider_name(), "Google Domains");
    /// ```
    ///
    /// # Returns
    ///
    /// The provider's human-readable name, `"Google Domains"`.
    fn provider_name(&self) -> &str {
        "Google Domains"
    }
}