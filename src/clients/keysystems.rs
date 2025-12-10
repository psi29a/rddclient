use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

/// Key-Systems (RRPproxy) DNS client
/// Uses Key-Systems dynamic DNS API
pub struct KeysystemsClient {
    server: String,
    token: String,
}

impl KeysystemsClient {
    /// Construct a `KeysystemsClient` from a `Config`.
    ///
    /// The `password` field of `config` is treated as the required API token; `server` is optional
    /// and defaults to "https://dynamicdns.key-systems.net" when not provided.
    ///
    /// # Parameters
    ///
    /// - `config`: Configuration containing at minimum a `password` (the Key‑Systems token). If
    ///   `config.password` is missing, construction fails.
    ///
    /// # Returns
    ///
    /// `Ok` with a `KeysystemsClient` configured with the token from `config.password` and the server
    /// URL from `config.server` (or the default), or an `Err` if the token is missing.
    ///
    /// # Examples
    ///
    /// ```
    /// // Construct a config with a token and optional server, then create the client.
    /// let cfg = Config {
    ///     server: Some("https://dynamicdns.key-systems.net".to_string()),
    ///     password: Some("secret-token".to_string()),
    ///     ..Default::default()
    /// };
    /// let client = KeysystemsClient::new(&cfg).unwrap();
    /// assert_eq!(client.provider_name(), "Key-Systems");
    /// ```
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let token = config.password.as_ref()
            .ok_or("password (token) is required for Key-Systems")?
            .clone();
        
        let server = config.server.clone()
            .unwrap_or_else(|| "https://dynamicdns.key-systems.net".to_string());

        Ok(KeysystemsClient {
            server,
            token,
        })
    }
}

impl DnsClient for KeysystemsClient {
    /// Update the DNS A record for `hostname` to the provided `ip` using the Key-Systems dynamic DNS API.
    ///
    /// Sends a GET request to the Key-Systems `/nic/update` endpoint with the client token, hostname,
    /// and IP; interprets DynDNS-like response codes and returns an error with a descriptive message
    /// for any failure responses.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the update succeeded (`good` or `nochg` responses), `Err` with a descriptive message otherwise.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::IpAddr;
    /// // let client = KeysystemsClient::new(&config).unwrap();
    /// // client.update_record("host.example.com", "1.2.3.4".parse::<IpAddr>().unwrap()).unwrap();
    /// ```
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        log::info!("Updating {} with Key-Systems", hostname);

        // Key-Systems dynamic DNS endpoint
        let url = format!("{}/nic/update", self.server);

        let response = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .with_param("token", &self.token)
            .with_param("hostname", hostname)
            .with_param("myip", ip.to_string())
            .send()?;

        let status_code = response.status_code;
        let body = response.as_str()?.trim();

        log::debug!("Response status: {}, body: {}", status_code, body);

        if status_code != 200 {
            return Err(format!("HTTP error: {}", status_code).into());
        }

        // Key-Systems uses DynDNS-like response codes
        if body.starts_with("good") || body.starts_with("nochg") {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else if body.starts_with("badauth") {
            Err("Authentication failed - invalid token".into())
        } else if body.starts_with("notfqdn") {
            Err("Invalid hostname format".into())
        } else if body.starts_with("nohost") {
            Err("Hostname not found in your account".into())
        } else if body.starts_with("abuse") {
            Err("Account blocked for abuse".into())
        } else if body.starts_with("badagent") {
            Err("User agent blocked".into())
        } else {
            Err(format!("Update failed: {}", body).into())
        }
    }

    /// Ensures the client has a non-empty authentication token.
    ///
    /// Returns `Ok(())` when the token is present; returns an `Err` if the token is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = KeysystemsClient { server: "https://dynamicdns.key-systems.net".into(), token: "token".into() };
    /// assert!(client.validate_config().is_ok());
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.token.is_empty() {
            return Err("password (token) is required for Key-Systems".into());
        }
        Ok(())
    }

    /// Provider name for this client.
    ///
    /// # Returns
    ///
    /// `&str` equal to "Key-Systems".
    ///
    /// # Examples
    ///
    /// ```
    /// let client = KeysystemsClient { server: "https://dynamicdns.key-systems.net".into(), token: "token".into() };
    /// assert_eq!(client.provider_name(), "Key-Systems");
    /// ```
    fn provider_name(&self) -> &str {
        "Key-Systems"
    }
}