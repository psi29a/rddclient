use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;
use base64::{Engine as _, engine::general_purpose};

pub struct ChangeipClient {
    username: String,
    password: String,
    server: String,
}

impl ChangeipClient {
    /// Creates a new `ChangeipClient` from the given configuration, validating required credentials.
    ///
    /// The `server` field in the configuration defaults to `"nic.changeip.com"` when not provided.
    ///
    /// # Errors
    ///
    /// Returns an `Err` if the configuration does not include a username or password.
    ///
    /// # Examples
    ///
    /// ```
    /// // Assuming `Config` has `login`, `password`, and optional `server` fields:
    /// let config = Config {
    ///     login: Some("user".into()),
    ///     password: Some("pass".into()),
    ///     server: None,
    ///     ..Default::default()
    /// };
    /// let client = ChangeipClient::new(&config).unwrap();
    /// assert_eq!(client.provider_name(), "ChangeIP");
    /// ```
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let username = config.login.as_ref()
            .ok_or("username is required for ChangeIP")?;
        let password = config.password.as_ref()
            .ok_or("password is required for ChangeIP")?;
        let server = config.server.as_deref()
            .unwrap_or("nic.changeip.com");

        Ok(ChangeipClient {
            username: username.to_string(),
            password: password.to_string(),
            server: server.to_string(),
        })
    }
}

impl DnsClient for ChangeipClient {
    /// Update the DNS A record for `hostname` at ChangeIP to the provided IP address.
    ///
    /// Sends an authenticated request to the ChangeIP API and interprets the JSON response:
    /// - returns `Ok(())` when the API indicates success or that the record was already unaltered;
    /// - returns `Err` with the provider error message when the API reports failure;
    /// - returns `Err` if the HTTP response code is not 200 or the response body is unexpected.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::IpAddr;
    /// // Construct a client (example fields); in real usage obtain via `ChangeipClient::new`.
    /// let client = ChangeipClient {
    ///     username: "user".into(),
    ///     password: "pass".into(),
    ///     server: "nic.changeip.com".into(),
    /// };
    /// let ip: IpAddr = "203.0.113.42".parse().unwrap();
    /// let _ = client.update_record("example.com", ip);
    /// ```
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        log::info!("Updating ChangeIP record for {} to {}", hostname, ip);

        let url = format!(
            "https://{}/nic/update?hostname={}&myip={}",
            self.server, hostname, ip
        );

        let auth = general_purpose::STANDARD.encode(format!("{}:{}", self.username, self.password));
        let response = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .with_header("Authorization", format!("Basic {}", auth))
            .send()?;

        if response.status_code != 200 {
            return Err(format!("ChangeIP API error: HTTP {}", response.status_code).into());
        }

        let body = response.as_str()?;
        
        // ChangeIP returns JSON response
        if body.contains("\"ok\":true") || body.contains("\"msg\":\"unaltered\"") {
            if body.contains("unaltered") {
                log::info!("IP address already set to {}", ip);
            } else {
                log::info!("Successfully updated DNS record for {} to {}", hostname, ip);
            }
            Ok(())
        } else if body.contains("\"ok\":false") {
            let error_msg = body
                .split("\"msg\":\"")
                .nth(1)
                .and_then(|s| s.split("\"").next())
                .unwrap_or("Unknown error");
            Err(format!("ChangeIP error: {}", error_msg).into())
        } else {
            Err(format!("Unexpected ChangeIP response: {}", body).into())
        }
    }

    /// Validates that the client's username and password are present.
    ///
    /// Returns `Ok(())` if both `username` and `password` are non-empty, `Err` with a message otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = ChangeipClient {
    ///     username: "user".to_string(),
    ///     password: "pass".to_string(),
    ///     server: "nic.changeip.com".to_string(),
    /// };
    /// assert!(client.validate_config().is_ok());
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.username.is_empty() {
            return Err("ChangeIP username cannot be empty".into());
        }
        if self.password.is_empty() {
            return Err("ChangeIP password cannot be empty".into());
        }
        Ok(())
    }

    /// Provider name for this DNS client.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = crate::clients::changeip::ChangeipClient {
    ///     username: String::new(),
    ///     password: String::new(),
    ///     server: "nic.changeip.com".into(),
    /// };
    /// assert_eq!(client.provider_name(), "ChangeIP");
    /// ```
    fn provider_name(&self) -> &'static str {
        "ChangeIP"
    }
}