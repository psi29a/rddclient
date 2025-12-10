use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;
use base64::{Engine as _, engine::general_purpose};

/// Loopia DNS client
/// Uses DynDNS2 protocol with Loopia's server
pub struct LoopiaClient {
    server: String,
    username: String,
    password: String,
}

impl LoopiaClient {
    /// Create a `LoopiaClient` from a configuration.
    ///
    /// The function reads `login` and `password` from `config` and returns an error if either is
    /// missing. If `server` is not provided in the configuration, it defaults to `https://dns.loopia.se`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use your_crate::clients::loopia::LoopiaClient;
    /// # use your_crate::config::Config;
    /// // Build a Config with `login`, `password`, and optionally `server`, then:
    /// // let client = LoopiaClient::new(&config).expect("valid Loopia config");
    /// ```
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let username = config.login.as_ref()
            .ok_or("username is required for Loopia")?
            .clone();
        let password = config.password.as_ref()
            .ok_or("password is required for Loopia")?
            .clone();
        
        let server = config.server.as_ref()
            .cloned()
            .unwrap_or_else(|| "https://dns.loopia.se".to_string());

        Ok(LoopiaClient {
            server,
            username,
            password,
        })
    }
}

impl DnsClient for LoopiaClient {
    /// Update the DNS record for `hostname` at Loopia using the DynDNS2 API.
    ///
    /// Attempts to set the host's IP to `ip` by calling Loopia's XDynDNS endpoint and
    /// interpreting DynDNS2-style responses.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success; an `Err` containing a descriptive error message otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::IpAddr;
    ///
    /// let client = LoopiaClient {
    ///     server: "https://dns.loopia.se".into(),
    ///     username: "user".into(),
    ///     password: "pass".into(),
    /// };
    ///
    /// let ip: IpAddr = "127.0.0.1".parse().unwrap();
    /// let _ = client.update_record("example.com", ip);
    /// ```
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let url = format!(
            "{}/XDynDNSServer/XDynDNS.php?hostname={}&myip={}",
            self.server, hostname, ip
        );

        log::info!("Updating {} with Loopia", hostname);

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

    /// Validates that the client has both username and password configured.
    ///
    /// Returns `Ok(())` if both `username` and `password` are non-empty, `Err` with a descriptive message otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// let good = LoopiaClient {
    ///     server: "https://dns.loopia.se".into(),
    ///     username: "user".into(),
    ///     password: "pass".into(),
    /// };
    /// assert!(good.validate_config().is_ok());
    ///
    /// let bad_user = LoopiaClient {
    ///     server: "https://dns.loopia.se".into(),
    ///     username: "".into(),
    ///     password: "pass".into(),
    /// };
    /// assert_eq!(bad_user.validate_config().unwrap_err().to_string(), "username is required for Loopia");
    ///
    /// let bad_pass = LoopiaClient {
    ///     server: "https://dns.loopia.se".into(),
    ///     username: "user".into(),
    ///     password: "".into(),
    /// };
    /// assert_eq!(bad_pass.validate_config().unwrap_err().to_string(), "password is required for Loopia");
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.username.is_empty() {
            return Err("username is required for Loopia".into());
        }
        if self.password.is_empty() {
            return Err("password is required for Loopia".into());
        }
        Ok(())
    }

    /// Returns the provider name for this client.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = LoopiaClient {
    ///     server: "https://dns.loopia.se".to_string(),
    ///     username: "user".to_string(),
    ///     password: "pass".to_string(),
    /// };
    /// assert_eq!(client.provider_name(), "Loopia");
    /// ```
    fn provider_name(&self) -> &str {
        "Loopia"
    }
}