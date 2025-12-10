use crate::clients::DnsClient;
use crate::config::Config;
use base64::{Engine as _, engine::general_purpose};
use std::error::Error;
use std::net::IpAddr;

pub struct EasydnsClient {
    username: String,
    password: String,
    server: String,
}

impl EasydnsClient {
    /// Create a new EasyDNS client from the provided configuration.
    ///
    /// The function reads `login` and `password` from `config` (both required) and uses
    /// `config.server` if present or defaults to `https://api.easydns.com` otherwise.
    /// Returns an error if `login` or `password` is missing.
    ///
    /// # Examples
    ///
    /// ```
    /// // Construct a Config with required fields (fields shown as in-crate types)
    /// let cfg = crate::Config {
    ///     login: Some("user@example.com".to_string()),
    ///     password: Some("s3cr3t".to_string()),
    ///     server: None,
    /// };
    /// let client = crate::clients::easydns::EasydnsClient::new(&cfg).unwrap();
    /// assert_eq!(client.provider_name(), "EasyDNS");
    /// ```
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let username = config.login.as_ref()
            .ok_or("EasyDNS requires username")?
            .clone();
        let password = config.password.as_ref()
            .ok_or("EasyDNS requires password")?
            .clone();
        let server = config.server.clone()
            .unwrap_or_else(|| "https://api.easydns.com".to_string());

        Ok(Self {
            username,
            password,
            server,
        })
    }
}

impl DnsClient for EasydnsClient {
    /// Updates the DNS A record for `hostname` at EasyDNS to the given `ip`.
    ///
    /// Interprets EasyDNS response bodies and HTTP status codes to determine success:
    /// returns `Err` when authentication fails, the hostname does not exist, the provider
    /// reports an error, or the HTTP response code is not 200; returns `Ok(())` when the
    /// update is accepted (`OK`, `good`, `nochg`) or when no explicit error is present.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the update is considered successful, `Err(...)` with a descriptive message otherwise.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::IpAddr;
    ///
    /// // Construct `config` appropriately for your environment before calling.
    /// let client = EasydnsClient::new(&config).expect("valid config");
    /// client.update_record("host.example.com", "1.2.3.4".parse::<IpAddr>().unwrap()).unwrap();
    /// ```
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let auth = general_purpose::STANDARD.encode(format!("{}:{}", self.username, self.password));
        
        let url = format!("{}/dyn/generic.php?hostname={}&myip={}", 
            self.server, hostname, ip);
        
        log::info!("Updating {} to {} (note: EasyDNS requires 10min between updates)", 
            hostname, ip);
        
        let response = minreq::get(&url)
            .with_header("Authorization", format!("Basic {}", auth))
            .with_header("User-Agent", crate::USER_AGENT)
            .send()?;

        if response.status_code != 200 {
            return Err(format!("HTTP error: {}", response.status_code).into());
        }

        let body = response.as_str()?;
        let body_lower = body.to_lowercase();
        
        // EasyDNS error codes
        if body_lower.contains("noaccess") || body_lower.contains("no_auth") {
            Err("Authentication failed (wrong username/password or host/domain)".into())
        } else if body_lower.contains("nohost") {
            Err("Hostname does not exist".into())
        } else if body_lower.contains("error") {
            Err(format!("Update failed: {}", body).into())
        } else if body.contains("OK") || body.contains("good") || body.contains("nochg") {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else {
            // Assume success if no error
            log::info!("Updated {} to {}", hostname, ip);
            Ok(())
        }
    }

    /// Ensures the client has non-empty credentials required by EasyDNS.
    ///
    /// Returns `Ok(())` if both `username` and `password` are not empty, `Err` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = EasydnsClient {
    ///     username: "user".into(),
    ///     password: "pass".into(),
    ///     server: "https://api.easydns.com".into(),
    /// };
    /// assert!(client.validate_config().is_ok());
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.username.is_empty() {
            return Err("EasyDNS username cannot be empty".into());
        }
        if self.password.is_empty() {
            return Err("EasyDNS password cannot be empty".into());
        }
        Ok(())
    }

    /// Provider name for this client.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = EasydnsClient { username: String::new(), password: String::new(), server: String::from("https://api.easydns.com") };
    /// assert_eq!(client.provider_name(), "EasyDNS");
    /// ```
    fn provider_name(&self) -> &str {
        "EasyDNS"
    }
}