use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;
use base64::{Engine as _, engine::general_purpose};

/// 1984.is DNS client
/// Uses DynDNS2 protocol with basic authentication
pub struct One984Client {
    server: String,
    username: String,
    password: String,
}

impl One984Client {
    /// Construct a One984Client by extracting credentials and server URL from the provided config.
    ///
    /// Returns an error if the config is missing a login or password. If `server` is not set in the
    /// config, the default "https://www.1984.is" is used.
    ///
    /// # Examples
    ///
    /// ```
    /// let config = Config {
    ///     login: Some("user".to_string()),
    ///     password: Some("secret".to_string()),
    ///     server: None,
    ///     ..Default::default()
    /// };
    /// let client = One984Client::new(&config).unwrap();
    /// assert_eq!(client.provider_name(), "1984.is");
    /// ```
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let username = config.login.as_ref()
            .ok_or("username is required for 1984.is")?
            .clone();
        let password = config.password.as_ref()
            .ok_or("password is required for 1984.is")?
            .clone();
        
        let server = config.server.as_ref()
            .cloned()
            .unwrap_or_else(|| "https://www.1984.is".to_string());

        Ok(One984Client {
            server,
            username,
            password,
        })
    }
}

impl DnsClient for One984Client {
    /// Updates the DNS A record for a hostname at the 1984.is DynDNS2 endpoint.
    ///
    /// Sends an authenticated DynDNS2-style GET request to the provider to set the host's IP
    /// to `ip`. Logs request/response details and maps common DynDNS2 responses to errors.
    ///
    /// # Parameters
    ///
    /// - `hostname`: The DNS hostname to update (fully qualified).
    /// - `ip`: The IPv4 or IPv6 address to assign to `hostname`.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the provider reports a successful update (`good` or `nochg`).
    /// `Err` when the HTTP request fails or the provider returns an error such as
    /// authentication failure, invalid hostname, unknown host, account abuse, server error,
    /// or any unexpected response body.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::net::IpAddr;
    /// # use crate::clients::one984::One984Client;
    /// # fn example(client: &One984Client) -> Result<(), Box<dyn std::error::Error>> {
    /// let ip: IpAddr = "203.0.113.5".parse()?;
    /// client.update_record("host.example.com", ip)?;
    /// # Ok(())
    /// # }
    /// ```
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let url = format!(
            "{}/nic/update?hostname={}&myip={}",
            self.server, hostname, ip
        );

        log::info!("Updating {} with 1984.is", hostname);

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

    /// Validates that the client's username and password are set.
    ///
    /// Returns `Ok(())` if both `username` and `password` are non-empty.
    /// Returns `Err` with a descriptive message if either field is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = crate::clients::one984::One984Client {
    ///     server: "https://www.1984.is".to_string(),
    ///     username: "user".to_string(),
    ///     password: "pass".to_string(),
    /// };
    /// assert!(client.validate_config().is_ok());
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.username.is_empty() {
            return Err("username is required for 1984.is".into());
        }
        if self.password.is_empty() {
            return Err("password is required for 1984.is".into());
        }
        Ok(())
    }

    /// Provider identifier for this DNS client.
    ///
    /// # Returns
    ///
    /// `"1984.is"`
    ///
    /// # Examples
    ///
    /// ```
    /// let client = One984Client { server: "https://example".into(), username: "user".into(), password: "pass".into() };
    /// assert_eq!(client.provider_name(), "1984.is");
    /// ```
    fn provider_name(&self) -> &str {
        "1984.is"
    }
}