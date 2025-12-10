use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;
use base64::{Engine as _, engine::general_purpose};

/// DynDNS v1 DNS client (legacy protocol)
/// Uses the original DynDNS v1 protocol (predates DynDNS2)
pub struct Dyndns1Client {
    server: String,
    username: String,
    password: String,
    static_ip: bool,
}

impl Dyndns1Client {
    /// Create a configured Dyndns1Client from the provided Config.
    ///
    /// Extracts `login` and `password` from `config` and uses `config.server` or
    /// defaults to "https://members.dyndns.org" when omitted. Returns an error if
    /// the required credentials are missing.
    ///
    /// # Parameters
    ///
    /// - `config`: configuration containing `login` and `password`; `server` is optional.
    ///
    /// # Returns
    ///
    /// A `Dyndns1Client` configured with the provided credentials and server, or an error if
    /// `login` or `password` is not present.
    ///
    /// # Examples
    ///
    /// ```
    /// let cfg = Config {
    ///     login: Some("user".into()),
    ///     password: Some("pass".into()),
    ///     server: None,
    ///     ..Default::default()
    /// };
    /// let client = Dyndns1Client::new(&cfg).expect("valid config");
    /// ```
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let username = config.login.as_ref()
            .ok_or("username is required for DynDNS v1")?
            .clone();
        
        let password = config.password.as_ref()
            .ok_or("password is required for DynDNS v1")?
            .clone();
        
        let server = config.server.clone()
            .unwrap_or_else(|| "https://members.dyndns.org".to_string());
        
        // Static IP flag for legacy DynDNS
        let static_ip = false;

        Ok(Dyndns1Client {
            server,
            username,
            password,
            static_ip,
        })
    }
}

impl DnsClient for Dyndns1Client {
    /// Update the DNS record for a hostname using the DynDNS v1 protocol.
    ///
    /// Performs an authenticated update request to the configured DynDNS v1 server for `hostname` with the given `ip`.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the update succeeded; `Err` with an error describing the failure otherwise.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Example usage (networking calls are ignored in doctests)
    /// let client = /* obtain a configured Dyndns1Client instance */;
    /// client.update_record("example.dyndns.org", "1.2.3.4".parse().unwrap())?;
    /// ```
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        log::info!("Updating {} with DynDNS v1", hostname);

        // DynDNS v1 update endpoint
        let url = format!("{}/nic/update", self.server);

        let auth = format!("{}:{}", self.username, self.password);
        let encoded_auth = format!("Basic {}", general_purpose::STANDARD.encode(auth.as_bytes()));

        let mut request = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .with_header("Authorization", &encoded_auth)
            .with_param("hostname", hostname)
            .with_param("myip", ip.to_string());

        // Add system parameter for static IPs (DynDNS v1 specific)
        if self.static_ip {
            request = request.with_param("system", "statdns");
        } else {
            request = request.with_param("system", "dyndns");
        }

        let response = request.send()?;

        let status_code = response.status_code;
        let body = response.as_str()?.trim();

        log::debug!("Response status: {}, body: {}", status_code, body);

        if status_code != 200 {
            return Err(format!("HTTP error: {}", status_code).into());
        }

        // DynDNS v1 protocol response codes
        if body.starts_with("good") || body.starts_with("nochg") {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else if body.starts_with("badauth") {
            Err("Authentication failed".into())
        } else if body.starts_with("notfqdn") {
            Err("Invalid hostname".into())
        } else if body.starts_with("nohost") {
            Err("Hostname not found".into())
        } else if body.starts_with("!donator") {
            Err("Feature requires donator account".into())
        } else if body.starts_with("!active") {
            Err("Hostname not activated".into())
        } else if body.starts_with("abuse") {
            Err("Hostname blocked for abuse".into())
        } else {
            Err(format!("Update failed: {}", body).into())
        }
    }

    /// Validates that the client's configuration contains both username and password.
    ///
    /// Returns `Ok(())` if both username and password are non-empty; otherwise returns an `Err` with a message
    /// indicating the missing field ("username is required for DynDNS v1" or "password is required for DynDNS v1").
    ///
    /// # Examples
    ///
    /// ```
    /// // After creating a client (e.g. via Dyndns1Client::new), call validate_config to ensure credentials are present.
    /// // let client = Dyndns1Client::new(&config).unwrap();
    /// // client.validate_config().unwrap();
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.username.is_empty() {
            return Err("username is required for DynDNS v1".into());
        }
        if self.password.is_empty() {
            return Err("password is required for DynDNS v1".into());
        }
        Ok(())
    }

    /// Human-readable provider name for this client.
    ///
    /// Returns the provider name string: `"DynDNS v1"`.
    ///
    /// # Examples
    ///
    /// ```
    /// // The provider identifier used by this client implementation.
    /// let name = "DynDNS v1";
    /// assert_eq!(name, "DynDNS v1");
    /// ```
    fn provider_name(&self) -> &str {
        "DynDNS v1"
    }
}