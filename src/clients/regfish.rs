use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;
use base64::{Engine as _, engine::general_purpose};

/// Regfish DNS client
/// Uses DynDNS2 protocol with Regfish's server
pub struct RegfishClient {
    server: String,
    username: String,
    password: String,
}

impl RegfishClient {
    /// Constructs a RegfishClient from a Config, requiring Regfish login credentials.
    ///
    /// The provided `config` must include `login` and `password`; if either is missing this function
    /// returns an error. If `config.server` is not set, the Regfish default `https://dyndns.regfish.de`
    /// will be used.
    ///
    /// # Errors
    ///
    /// Returns an `Err` if `login` or `password` are not present in `config`.
    ///
    /// # Examples
    ///
    /// ```
    /// let cfg = Config {
    ///     login: Some("user".to_string()),
    ///     password: Some("secret".to_string()),
    ///     server: None,
    ///     ..Default::default()
    /// };
    /// let client = RegfishClient::new(&cfg).unwrap();
    /// assert_eq!(client.provider_name(), "Regfish");
    /// ```
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let username = config.login.as_ref()
            .ok_or("username is required for Regfish")?
            .clone();
        let password = config.password.as_ref()
            .ok_or("password is required for Regfish")?
            .clone();
        
        let server = config.server.as_ref()
            .cloned()
            .unwrap_or_else(|| "https://dyndns.regfish.de".to_string());

        Ok(RegfishClient {
            server,
            username,
            password,
        })
    }
}

impl DnsClient for RegfishClient {
    /// Update the DNS A record for `hostname` to `ip` using Regfish's DynDNS2-compatible API.
    ///
    /// Sends an HTTP GET to the configured Regfish server and interprets DynDNS2-style responses
    /// to determine success or the specific failure reason.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP status is not 200, or when the Regfish response indicates:
    /// - `badauth`: authentication failed
    /// - `notfqdn`: invalid hostname format
    /// - `nohost`: hostname does not exist
    /// - `abuse`: account blocked for abuse
    /// - `911`: server-side error (try again later)
    /// - any other unexpected response body
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::net::IpAddr;
    /// # let client = /* RegfishClient constructed elsewhere */ unimplemented!();
    /// # let hostname = "example.example.com";
    /// # let ip: IpAddr = "1.2.3.4".parse().unwrap();
    /// let _ = client.update_record(hostname, ip);
    /// ```
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let url = format!(
            "{}/?fqdn={}&forcehost=1&authtype=secure&token={}",
            self.server, hostname, self.password
        );

        log::info!("Updating {} with Regfish", hostname);

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

    /// Validate that the client has the required credentials configured.
    ///
    /// Ensures both `username` and `password` are non-empty.
    ///
    /// # Returns
    ///
    /// `Ok(())` if both `username` and `password` are non-empty, `Err` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::IpAddr;
    ///
    /// // Construct a RegfishClient with credentials and validate them.
    /// let client = RegfishClient {
    ///     server: "https://dyndns.regfish.de".into(),
    ///     username: "user".into(),
    ///     password: "pass".into(),
    /// };
    /// assert!(client.validate_config().is_ok());
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.username.is_empty() {
            return Err("username is required for Regfish".into());
        }
        if self.password.is_empty() {
            return Err("password is required for Regfish".into());
        }
        Ok(())
    }

    /// Provider identifier for this client.
    ///
    /// Returns: `&str` with the provider name, `Regfish`.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = RegfishClient {
    ///     server: String::from("https://dyndns.regfish.de"),
    ///     username: String::from("user"),
    ///     password: String::from("pass"),
    /// };
    /// assert_eq!(client.provider_name(), "Regfish");
    /// ```
    fn provider_name(&self) -> &str {
        "Regfish"
    }
}