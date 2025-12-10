use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;
use base64::{Engine as _, engine::general_purpose};

/// ZoneEdit v1 DNS client (legacy protocol)
/// Uses ZoneEdit's legacy dynamic DNS protocol
pub struct Zoneedit1Client {
    server: String,
    username: String,
    password: String,
}

impl Zoneedit1Client {
    /// Constructs a Zoneedit1Client from configuration, requiring a username and password and defaulting the server if unset.
    ///
    /// Uses `config.login` and `config.password` as the credentials; if either is missing an error is returned. If `config.server` is not provided, the server URL defaults to "https://dynamic.zoneedit.com".
    ///
    /// # Returns
    ///
    /// A configured `Zoneedit1Client` on success, or an error if `login` or `password` are missing.
    ///
    /// # Examples
    ///
    /// ```
    /// let config = crate::Config { login: Some("user".into()), password: Some("pass".into()), server: None, ..Default::default() };
    /// let client = crate::clients::zoneedit1::Zoneedit1Client::new(&config).unwrap();
    /// assert_eq!(client.provider_name(), "ZoneEdit v1");
    /// ```
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let username = config.login.as_ref()
            .ok_or("username is required for ZoneEdit v1")?
            .clone();
        
        let password = config.password.as_ref()
            .ok_or("password is required for ZoneEdit v1")?
            .clone();
        
        let server = config.server.clone()
            .unwrap_or_else(|| "https://dynamic.zoneedit.com".to_string());

        Ok(Zoneedit1Client {
            server,
            username,
            password,
        })
    }
}

impl DnsClient for Zoneedit1Client {
    /// Updates the DNS A record for `hostname` to the given `ip` using the ZoneEdit v1 dynamic update API.
    ///
    /// Sends an authenticated request to the ZoneEdit v1 endpoint and interprets the HTML response for success or specific error codes; returns `Ok(())` on successful update and `Err` with a descriptive message on failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::IpAddr;
    ///
    /// let client = Zoneedit1Client {
    ///     server: "https://dynamic.zoneedit.com".into(),
    ///     username: "user".into(),
    ///     password: "pass".into(),
    /// };
    ///
    /// // Call the update; in real usage handle the Result appropriately.
    /// let _ = client.update_record("example.com", "1.2.3.4".parse::<IpAddr>().unwrap());
    /// ```
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        log::info!("Updating {} with ZoneEdit v1", hostname);

        // ZoneEdit v1 update endpoint
        let url = format!("{}/auth/dynamic.html", self.server);

        let auth = format!("{}:{}", self.username, self.password);
        let encoded_auth = format!("Basic {}", general_purpose::STANDARD.encode(auth.as_bytes()));

        let response = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .with_header("Authorization", &encoded_auth)
            .with_param("host", hostname)
            .with_param("dnsto", ip.to_string())
            .send()?;

        let status_code = response.status_code;
        let body = response.as_str()?.trim();

        log::debug!("Response status: {}, body: {}", status_code, body);

        if status_code != 200 {
            return Err(format!("HTTP error: {}", status_code).into());
        }

        // ZoneEdit v1 returns HTML with status indicators
        if body.contains("SUCCESS") || body.contains("UPDATE") {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else if body.contains("ERROR CODE=\"707\"") {
            Err("Update failed - duplicate update".into())
        } else if body.contains("ERROR CODE=\"701\"") {
            Err("Zone not found".into())
        } else if body.contains("ERROR CODE=\"702\"") {
            Err("Record not found".into())
        } else if body.contains("ERROR") {
            Err(format!("ZoneEdit v1 error: {}", body).into())
        } else if body.contains("INVALID_USER") || body.contains("INVALID_PASS") {
            Err("Authentication failed".into())
        } else {
            Err(format!("Unexpected response: {}", body).into())
        }
    }

    /// Validates that the client has both username and password configured.
    ///
    /// # Errors
    ///
    /// Returns `Err` if `username` is empty with the message `"username is required for ZoneEdit v1"`,
    /// or if `password` is empty with the message `"password is required for ZoneEdit v1"`.
    ///
    /// # Examples
    ///
    /// ```
    /// let ok = Zoneedit1Client {
    ///     server: "https://dynamic.zoneedit.com".into(),
    ///     username: "user".into(),
    ///     password: "pass".into(),
    /// };
    /// assert!(ok.validate_config().is_ok());
    ///
    /// let no_user = Zoneedit1Client {
    ///     server: "https://dynamic.zoneedit.com".into(),
    ///     username: "".into(),
    ///     password: "pass".into(),
    /// };
    /// assert!(no_user.validate_config().is_err());
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.username.is_empty() {
            return Err("username is required for ZoneEdit v1".into());
        }
        if self.password.is_empty() {
            return Err("password is required for ZoneEdit v1".into());
        }
        Ok(())
    }

    /// Provider identifier for this DNS client.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = crate::clients::zoneedit1::Zoneedit1Client {
    ///     server: String::new(),
    ///     username: String::new(),
    ///     password: String::new(),
    /// };
    /// assert_eq!(client.provider_name(), "ZoneEdit v1");
    /// ```
    fn provider_name(&self) -> &str {
        "ZoneEdit v1"
    }
}