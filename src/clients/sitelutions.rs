use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;
use base64::{Engine as _, engine::general_purpose};

/// Sitelutions DNS client
/// Uses DynDNS2-style protocol with Sitelutions' server
pub struct SitelutionsClient {
    server: String,
    username: String,
    password: String,
}

impl SitelutionsClient {
    /// Constructs a SitelutionsClient from a Config.
    ///
    /// The function requires `login` and `password` to be present in `config`; if either is missing it returns an error with the message
    /// "username is required for Sitelutions" or "password is required for Sitelutions" respectively. If `server` is not provided, it defaults
    /// to "https://www.sitelutions.com".
    ///
    /// # Parameters
    ///
    /// - `config`: Configuration containing `login`, `password`, and an optional `server` URL.
    ///
    /// # Returns
    ///
    /// `Ok(SitelutionsClient)` when credentials are present and the client is created, or an `Err` with a descriptive message when `login` or `password` is missing.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let config = Config {
    ///     login: Some("user".to_string()),
    ///     password: Some("pass".to_string()),
    ///     server: None,
    ///     ..Default::default()
    /// };
    /// let client = SitelutionsClient::new(&config).unwrap();
    /// assert_eq!(client.provider_name(), "Sitelutions");
    /// ```
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let username = config.login.as_ref()
            .ok_or("username is required for Sitelutions")?
            .clone();
        let password = config.password.as_ref()
            .ok_or("password is required for Sitelutions")?
            .clone();
        
        let server = config.server.as_ref()
            .cloned()
            .unwrap_or_else(|| "https://www.sitelutions.com".to_string());

        Ok(SitelutionsClient {
            server,
            username,
            password,
        })
    }
}

impl DnsClient for SitelutionsClient {
    /// Sends a DynDNS2-style update for `hostname` to the Sitelutions service and interprets the response.
    ///
    /// The method constructs an update request to the configured server, authenticates with HTTP Basic
    /// using the client's credentials, and interprets DynDNS2-style response bodies:
    /// "good"/"nochg" indicate success; "badauth", "notfqdn", "nohost", "abuse", "911" and other bodies
    /// are mapped to descriptive errors.
    ///
    /// # Returns
    ///
    /// `Ok(())` on successful update; `Err` with a descriptive message for HTTP errors or any recognized
    /// or unexpected service response.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::IpAddr;
    /// let client = SitelutionsClient {
    ///     server: "https://www.sitelutions.com".to_string(),
    ///     username: "user".to_string(),
    ///     password: "pass".to_string(),
    /// };
    /// let _ = client.update_record("example.example", "203.0.113.1".parse::<IpAddr>().unwrap());
    /// ```
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let url = format!(
            "{}/dnsup?hostname={}&ip={}",
            self.server, hostname, ip
        );

        log::info!("Updating {} with Sitelutions", hostname);

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

    /// Ensures the client has both a username and password configured.
    ///
    /// Returns `Ok(())` if both `username` and `password` are non-empty.
    /// Returns an `Err` with the message `"username is required for Sitelutions"` if `username` is empty,
    /// or `"password is required for Sitelutions"` if `password` is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = SitelutionsClient {
    ///     server: "https://www.sitelutions.com".into(),
    ///     username: "user".into(),
    ///     password: "pass".into(),
    /// };
    /// assert!(client.validate_config().is_ok());
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.username.is_empty() {
            return Err("username is required for Sitelutions".into());
        }
        if self.password.is_empty() {
            return Err("password is required for Sitelutions".into());
        }
        Ok(())
    }

    /// Provider name for this client.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = SitelutionsClient {
    ///     server: String::from("https://www.sitelutions.com"),
    ///     username: String::from("user"),
    ///     password: String::from("pass"),
    /// };
    /// assert_eq!(client.provider_name(), "Sitelutions");
    /// ```
    fn provider_name(&self) -> &str {
        "Sitelutions"
    }
}