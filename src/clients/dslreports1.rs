use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

/// DSLReports DNS client (legacy v1 protocol)
/// Uses DSLReports legacy update protocol
pub struct Dslreports1Client {
    server: String,
    username: String,
    password: String,
}

impl Dslreports1Client {
    /// Create a new DSLReports v1 client from the provided configuration.
    ///
    /// The `config` must include `login` and `password`; the function returns an error if either is missing. If `server` is not set in the config, the default "https://www.dslreports.com" is used.
    ///
    /// # Parameters
    ///
    /// - `config`: configuration containing credentials and optional server URL.
    ///
    /// # Returns
    ///
    /// A configured `Dslreports1Client` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// // Assuming `Config` is in scope with fields `login`, `password`, and `server`.
    /// let cfg = Config {
    ///     login: Some("user".into()),
    ///     password: Some("pass".into()),
    ///     server: None,
    ///     ..Default::default()
    /// };
    /// let client = Dslreports1Client::new(&cfg).unwrap();
    /// assert_eq!(client.provider_name(), "DSLReports");
    /// ```
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let username = config.login.as_ref()
            .ok_or("username is required for DSLReports")?
            .clone();
        
        let password = config.password.as_ref()
            .ok_or("password is required for DSLReports")?
            .clone();
        
        let server = config.server.clone()
            .unwrap_or_else(|| "https://www.dslreports.com".to_string());

        Ok(Dslreports1Client {
            server,
            username,
            password,
        })
    }
}

impl DnsClient for Dslreports1Client {
    /// Update the DNS A record for `hostname` at DSLReports using the legacy v1 endpoint.
    ///
    /// Sends a GET request to the client's `{server}/updateip` endpoint with the configured
    /// username/password, the target `hostname`, and `ip`, and interprets the provider's
    /// plain-text response to determine success.
    ///
    /// # Returns
    /// `Ok(())` when the update is reported as successful.
    ///
    /// # Errors
    /// Returns an error when:
    /// - the HTTP status is not 200 (`HTTP error: {status}`),
    /// - the provider response contains `badauth` (`"Authentication failed"`),
    /// - the provider response contains `notfqdn` (`"Invalid hostname"`),
    /// - the provider response indicates any other failure (`"Update failed: {body}"`),
    /// - or when the HTTP client or response parsing fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::IpAddr;
    /// // assume `client` is a constructed Dslreports1Client
    /// // let client = Dslreports1Client::new(&config).unwrap();
    /// let ip: IpAddr = "1.2.3.4".parse().unwrap();
    /// let result = client.update_record("example.dslreports.test", ip);
    /// assert!(result.is_ok());
    /// ```
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        log::info!("Updating {} with DSLReports", hostname);

        // DSLReports legacy update endpoint
        let url = format!("{}/updateip", self.server);

        let response = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .with_param("username", &self.username)
            .with_param("password", &self.password)
            .with_param("hostname", hostname)
            .with_param("ip", ip.to_string())
            .send()?;

        let status_code = response.status_code;
        let body = response.as_str()?.trim();

        log::debug!("Response status: {}, body: {}", status_code, body);

        if status_code != 200 {
            return Err(format!("HTTP error: {}", status_code).into());
        }

        // DSLReports returns simple text responses
        if body.contains("good") || body.contains("successful") || body == "OK" {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else if body.contains("badauth") {
            Err("Authentication failed".into())
        } else if body.contains("notfqdn") {
            Err("Invalid hostname".into())
        } else {
            Err(format!("Update failed: {}", body).into())
        }
    }

    /// Validate that the client has both a username and a password configured.
    ///
    /// Returns `Ok(())` if both `username` and `password` are non-empty, `Err` otherwise with a message indicating the missing field.
    ///
    /// # Examples
    ///
    /// ```
    /// let ok = Dslreports1Client { server: "https://example.com".into(), username: "user".into(), password: "pass".into() };
    /// assert!(ok.validate_config().is_ok());
    ///
    /// let no_user = Dslreports1Client { server: "https://example.com".into(), username: "".into(), password: "pass".into() };
    /// assert!(no_user.validate_config().is_err());
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.username.is_empty() {
            return Err("username is required for DSLReports".into());
        }
        if self.password.is_empty() {
            return Err("password is required for DSLReports".into());
        }
        Ok(())
    }

    /// Returns the provider identifier for this client.
    ///
    /// # Examples
    ///
    /// ```
    /// let name = Dslreports1Client { server: String::new(), username: String::new(), password: String::new() }.provider_name();
    /// assert_eq!(name, "DSLReports");
    /// ```
    fn provider_name(&self) -> &str {
        "DSLReports"
    }
}