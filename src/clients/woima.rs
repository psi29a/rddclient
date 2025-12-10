use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

/// Woima.fi DNS client
/// Uses Woima.fi DynDNS2 protocol
pub struct WoimaClient {
    server: String,
    username: String,
    password: String,
}

impl WoimaClient {
    /// Creates a new WoimaClient from the provided configuration.
    ///
    /// Returns an error if the configuration does not contain a username or password.
    /// The server URL is taken from `config.server` when present; otherwise it defaults to
    /// "https://www.woima.fi".
    ///
    /// # Examples
    ///
    /// ```
    /// let cfg = Config {
    ///     login: Some("alice".to_string()),
    ///     password: Some("s3cr3t".to_string()),
    ///     server: None,
    ///     ..Default::default()
    /// };
    /// let client = WoimaClient::new(&cfg).expect("valid Woima configuration");
    /// assert_eq!(client.provider_name(), "Woima.fi");
    /// ```
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let username = config.login.as_ref()
            .ok_or("username is required for Woima.fi")?
            .clone();
        
        let password = config.password.as_ref()
            .ok_or("password is required for Woima.fi")?
            .clone();
        
        let server = config.server.clone()
            .unwrap_or_else(|| "https://www.woima.fi".to_string());

        Ok(WoimaClient {
            server,
            username,
            password,
        })
    }
}

impl DnsClient for WoimaClient {
    /// Update a DNS A record at Woima.fi using the DynDNS2-compatible API.
    ///
    /// The method performs an authenticated request to the provider's /nic/update
    /// endpoint and interprets DynDNS2 response codes to determine success or
    /// failure.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the provider accepted the update (response starts with `good` or `nochg`); `Err` with a descriptive message for authentication errors, invalid hostnames, missing hosts, account blocks, HTTP errors, or other provider responses.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::IpAddr;
    ///
    /// let client = WoimaClient {
    ///     server: "https://www.woima.fi".into(),
    ///     username: "user".into(),
    ///     password: "pass".into(),
    /// };
    ///
    /// // Attempt to update example.com to 1.2.3.4
    /// let ip: IpAddr = "1.2.3.4".parse().unwrap();
    /// let res = client.update_record("example.com", ip);
    /// assert!(res.is_ok() || res.is_err());
    /// ```
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        log::info!("Updating {} with Woima.fi", hostname);

        // Woima.fi DynDNS2 compatible endpoint
        let url = format!("{}/nic/update", self.server);

        let auth = format!("{}:{}", self.username, self.password);
        let encoded_auth = format!("Basic {}", base64::encode(&auth));

        let response = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .with_header("Authorization", &encoded_auth)
            .with_param("hostname", hostname)
            .with_param("myip", ip.to_string())
            .send()?;

        let status_code = response.status_code;
        let body = response.as_str()?.trim();

        log::debug!("Response status: {}, body: {}", status_code, body);

        if status_code != 200 {
            return Err(format!("HTTP error: {}", status_code).into());
        }

        // DynDNS2 protocol response codes
        if body.starts_with("good") || body.starts_with("nochg") {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else if body.starts_with("badauth") {
            Err("Authentication failed".into())
        } else if body.starts_with("notfqdn") {
            Err("Invalid hostname".into())
        } else if body.starts_with("nohost") {
            Err("Hostname not found".into())
        } else if body.starts_with("abuse") {
            Err("Account blocked for abuse".into())
        } else {
            Err(format!("Update failed: {}", body).into())
        }
    }

    /// Ensure the client has the required Woima.fi credentials.
    ///
    /// Returns `Ok(())` when both `username` and `password` are non-empty; returns `Err` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = crate::clients::woima::WoimaClient {
    ///     server: "https://www.woima.fi".into(),
    ///     username: "user".into(),
    ///     password: "secret".into(),
    /// };
    /// assert!(client.validate_config().is_ok());
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.username.is_empty() {
            return Err("username is required for Woima.fi".into());
        }
        if self.password.is_empty() {
            return Err("password is required for Woima.fi".into());
        }
        Ok(())
    }

    /// Provides the human-readable name of this DNS provider.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = WoimaClient {
    ///     server: "https://www.woima.fi".into(),
    ///     username: "user".into(),
    ///     password: "pass".into(),
    /// };
    /// assert_eq!(client.provider_name(), "Woima.fi");
    /// ```
    fn provider_name(&self) -> &str {
        "Woima.fi"
    }
}

mod base64 {
    use base64::{Engine as _, engine::general_purpose};
    
    /// Encodes a string using standard Base64 encoding.
    ///
    /// Returns the Base64-encoded representation of `data`.
    ///
    /// # Examples
    ///
    /// ```
    /// let out = crate::clients::woima::base64::encode("user:pass");
    /// assert_eq!(out, "dXNlcjpwYXNz");
    /// ```
    pub fn encode(data: &str) -> String {
        general_purpose::STANDARD.encode(data.as_bytes())
    }
}