use crate::clients::DnsClient;
use crate::config::Config;
use base64::{Engine as _, engine::general_purpose};
use std::error::Error;
use std::net::IpAddr;

pub struct DomeneshopClient {
    username: String,
    password: String,
    server: String,
}

impl DomeneshopClient {
    /// Creates a new DomeneshopClient from a configuration.
    ///
    /// The function extracts the required `login` (used as username) and `password` (used as API secret)
    /// from `config`, and uses `config.server` if present or `https://api.domeneshop.no` as a default.
    ///
    /// # Returns
    ///
    /// `Ok(DomeneshopClient)` when both username and password are present; `Err` containing a message
    /// when either the username (`login`) or password is missing.
    ///
    /// # Examples
    ///
    /// ```
    /// // let config = Config { login: Some("user".into()), password: Some("secret".into()), server: None };
    /// // let client = DomeneshopClient::new(&config).unwrap();
    /// ```
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let username = config.login.as_ref()
            .ok_or("Domeneshop requires username (API token)")?
            .clone();
        let password = config.password.as_ref()
            .ok_or("Domeneshop requires password (API secret)")?
            .clone();
        let server = config.server.clone()
            .unwrap_or_else(|| "https://api.domeneshop.no".to_string());

        Ok(Self {
            username,
            password,
            server,
        })
    }
}

impl DnsClient for DomeneshopClient {
    /// Updates the DNS A record for `hostname` to the provided `ip` using the Domeneshop DynDNS API.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the update succeeded (HTTP status 200 or 204 and response empty or containing `good`/`nochg`).
    /// `Err` when the request returned a non-200/204 status, when the response contains `badauth` (invalid credentials), `nohost` (hostname not found), or any other failure message.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::IpAddr;
    /// let client = DomeneshopClient {
    ///     username: "user".into(),
    ///     password: "pass".into(),
    ///     server: "https://api.domeneshop.no".into(),
    /// };
    /// client.update_record("example.com", "1.2.3.4".parse::<IpAddr>().unwrap()).unwrap();
    /// ```
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let auth = general_purpose::STANDARD.encode(format!("{}:{}", self.username, self.password));
        
        let url = format!("{}/v0/dyndns/update?hostname={}&myip={}", 
            self.server, hostname, ip);
        
        log::info!("Updating {} to {}", hostname, ip);
        
        let response = minreq::get(&url)
            .with_header("Authorization", format!("Basic {}", auth))
            .with_header("User-Agent", crate::USER_AGENT)
            .send()?;

        if response.status_code != 200 && response.status_code != 204 {
            return Err(format!("HTTP error: {}", response.status_code).into());
        }

        let body = response.as_str()?;
        
        // Check for success
        if body.is_empty() || body.contains("good") || body.contains("nochg") {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else if body.contains("badauth") {
            Err("Bad authorization (invalid credentials)".into())
        } else if body.contains("nohost") {
            Err("Hostname does not exist".into())
        } else {
            Err(format!("Update failed: {}", body).into())
        }
    }

    /// Validates that the client has the required credentials configured.
    ///
    /// Returns `Ok(())` when both username and password are non-empty, `Err` with a
    /// descriptive message otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = DomeneshopClient {
    ///     username: String::from("user"),
    ///     password: String::from("pass"),
    ///     server: String::from("https://api.domeneshop.no"),
    /// };
    /// assert!(client.validate_config().is_ok());
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.username.is_empty() {
            return Err("Domeneshop username cannot be empty".into());
        }
        if self.password.is_empty() {
            return Err("Domeneshop password cannot be empty".into());
        }
        Ok(())
    }

    /// Get the DNS provider's name for this client.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = DomeneshopClient { username: "u".into(), password: "p".into(), server: "https://api.domeneshop.no".into() };
    /// assert_eq!(client.provider_name(), "Domeneshop");
    /// ```
    fn provider_name(&self) -> &str {
        "Domeneshop"
    }
}