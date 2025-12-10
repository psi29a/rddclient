use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

/// DNS Made Easy client
/// Uses simplified API (full REST API with HMAC would be more complex)
pub struct DnsMadeEasyClient {
    server: String,
    username: String,
    password: String,
}

impl DnsMadeEasyClient {
    /// Constructs a `DnsMadeEasyClient` from configuration values.
    ///
    /// Requires `config.login` and `config.password`; if `config.server` is absent the
    /// default `"https://cp.dnsmadeeasy.com"` is used. Returns an error with the
    /// messages `"username is required for DNS Made Easy"` or
    /// `"password is required for DNS Made Easy"` when the respective fields are missing.
    ///
    /// # Examples
    ///
    /// ```
    /// // `Config` is expected to have `login`, `password`, and optional `server`.
    /// let cfg = Config {
    ///     login: Some("alice".into()),
    ///     password: Some("s3cr3t".into()),
    ///     server: None,
    /// };
    /// let client = DnsMadeEasyClient::new(&cfg).unwrap();
    /// assert_eq!(client.provider_name(), "DNS Made Easy");
    /// ```
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let username = config.login.as_ref()
            .ok_or("username is required for DNS Made Easy")?
            .clone();
        let password = config.password.as_ref()
            .ok_or("password is required for DNS Made Easy")?
            .clone();
        
        let server = config.server.as_ref()
            .cloned()
            .unwrap_or_else(|| "https://cp.dnsmadeeasy.com".to_string());

        Ok(DnsMadeEasyClient {
            server,
            username,
            password,
        })
    }
}

impl DnsClient for DnsMadeEasyClient {
    /// Updates the DNS A record for `hostname` to the provided `ip` using DNS Made Easy's dynamic update endpoint.
    ///
    /// Sends an HTTP GET to the provider's update URL, checks the HTTP status, and interprets the provider response to determine success.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = crate::clients::dnsmadeeasy::DnsMadeEasyClient {
    ///     server: "https://cp.dnsmadeeasy.com".into(),
    ///     username: "user".into(),
    ///     password: "pass".into(),
    /// };
    /// let ip = std::net::IpAddr::V4(std::net::Ipv4Addr::new(1, 2, 3, 4));
    /// let _ = client.update_record("example.com", ip);
    /// ```
    ///
    /// # Returns
    ///
    /// `Ok(())` on successful update; `Err` with a descriptive message for HTTP errors, provider-reported errors, or unexpected responses.
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        // DNS Made Easy dynamic DNS endpoint
        let url = format!(
            "{}/servlet/updateip?username={}&password={}&id={}&ip={}",
            self.server, self.username, self.password, hostname, ip
        );

        log::info!("Updating {} with DNS Made Easy", hostname);

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
        if body.contains("success") || body.contains("updated") || body == "good" {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else if body.contains("error") || body.contains("invalid") {
            Err(format!("DNS Made Easy error: {}", body).into())
        } else {
            Err(format!("Unexpected response: {}", body).into())
        }
    }

    /// Ensures the client has both a username and password configured.
    ///
    /// # Returns
    ///
    /// `Ok(())` if both `username` and `password` are non-empty, `Err` with a descriptive message otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = DnsMadeEasyClient {
    ///     server: "https://cp.dnsmadeeasy.com".into(),
    ///     username: "user".into(),
    ///     password: "pass".into(),
    /// };
    /// assert!(client.validate_config().is_ok());
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.username.is_empty() {
            return Err("username is required for DNS Made Easy".into());
        }
        if self.password.is_empty() {
            return Err("password is required for DNS Made Easy".into());
        }
        Ok(())
    }

    /// Provider display name for the DNS Made Easy client.
    ///
    /// # Returns
    ///
    /// `"DNS Made Easy"` — the provider name.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = crate::clients::dnsmadeeasy::DnsMadeEasyClient {
    ///     server: String::new(),
    ///     username: String::new(),
    ///     password: String::new(),
    /// };
    /// assert_eq!(client.provider_name(), "DNS Made Easy");
    /// ```
    fn provider_name(&self) -> &str {
        "DNS Made Easy"
    }
}