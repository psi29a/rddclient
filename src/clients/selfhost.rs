use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

/// Selfhost.de DNS client
/// Uses Selfhost.de DynDNS2 protocol
pub struct SelfhostClient {
    server: String,
    username: String,
    password: String,
}

impl SelfhostClient {
    /// Creates a new SelfhostClient from configuration, validating required credentials and applying a default server.
    ///
    /// The function requires `config.login` and `config.password` to be present; if either is missing an error is returned with a clear message. If `config.server` is not provided, the server defaults to "https://carol.selfhost.de".
    ///
    /// # Examples
    ///
    /// ```
    /// let cfg = Config {
    ///     login: Some("user".to_string()),
    ///     password: Some("secret".to_string()),
    ///     server: None,
    /// };
    /// let client = SelfhostClient::new(&cfg).unwrap();
    /// assert_eq!(client.server, "https://carol.selfhost.de");
    /// ```
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let username = config.login.as_ref()
            .ok_or("username is required for Selfhost.de")?
            .clone();
        
        let password = config.password.as_ref()
            .ok_or("password is required for Selfhost.de")?
            .clone();
        
        let server = config.server.clone()
            .unwrap_or_else(|| "https://carol.selfhost.de".to_string());

        Ok(SelfhostClient {
            server,
            username,
            password,
        })
    }
}

impl DnsClient for SelfhostClient {
    /// Update the DNS A/AAAA record for a hostname using Selfhost.de's DynDNS2 API.
    ///
    /// Sends a GET request to the Selfhost.de /nic/update endpoint with HTTP Basic
    /// authentication and checks both the HTTP status and the DynDNS2 protocol
    /// response to determine success or failure.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success; otherwise an `Err` describing the failure (HTTP status
    /// error or a DynDNS2 protocol error message).
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::IpAddr;
    /// let client = SelfhostClient {
    ///     server: "https://carol.selfhost.de".into(),
    ///     username: "user".into(),
    ///     password: "pass".into(),
    /// };
    /// let ip: IpAddr = "127.0.0.1".parse().unwrap();
    /// let result = client.update_record("example.com", ip);
    /// // In normal usage, check result for success or handle the error.
    /// let _ = result;
    /// ```
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        log::info!("Updating {} with Selfhost.de", hostname);

        // Selfhost.de DynDNS2 compatible endpoint
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

    /// Ensures the client has the required credentials configured.
    ///
    /// Checks that both `username` and `password` are non-empty and returns an error describing the missing field if either is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = SelfhostClient {
    ///     server: "https://carol.selfhost.de".into(),
    ///     username: "alice".into(),
    ///     password: "s3cr3t".into(),
    /// };
    /// assert!(client.validate_config().is_ok());
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.username.is_empty() {
            return Err("username is required for Selfhost.de".into());
        }
        if self.password.is_empty() {
            return Err("password is required for Selfhost.de".into());
        }
        Ok(())
    }

    /// DNS provider name for this client.
    ///
    /// The returned value is the static provider identifier "Selfhost.de".
    ///
    /// # Examples
    ///
    /// ```
    /// let client = SelfhostClient {
    ///     server: String::from("https://carol.selfhost.de"),
    ///     username: String::from("user"),
    ///     password: String::from("pass"),
    /// };
    /// assert_eq!(client.provider_name(), "Selfhost.de");
    /// ```
    fn provider_name(&self) -> &str {
        "Selfhost.de"
    }
}

mod base64 {
    use base64::{Engine as _, engine::general_purpose};
    
    /// Encodes the given string using standard Base64 (RFC 4648) encoding.
    ///
    /// # Examples
    ///
    /// ```
    /// let out = crate::clients::selfhost::base64::encode("hello");
    /// assert_eq!(out, "aGVsbG8=");
    /// ```
    pub fn encode(data: &str) -> String {
        general_purpose::STANDARD.encode(data.as_bytes())
    }
}