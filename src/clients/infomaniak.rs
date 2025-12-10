use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;
use base64::{Engine as _, engine::general_purpose};

/// Infomaniak DNS client
/// Uses Infomaniak's API with basic authentication
pub struct InfomaniakClient {
    server: String,
    username: String,
    password: String,
}

impl InfomaniakClient {
    /// Creates a new InfomaniakClient from configuration values.
    ///
    /// The `login` and `password` fields in `config` are required and will cause an error
    /// if missing. The `server` field is optional and defaults to `https://infomaniak.com`
    /// when not provided.
    ///
    /// # Errors
    ///
    /// Returns an `Err` if `config.login` or `config.password` is `None`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let cfg = Config {
    ///     login: Some("user@example.com".into()),
    ///     password: Some("s3cr3t".into()),
    ///     server: None,
    /// };
    /// let client = InfomaniakClient::new(&cfg).unwrap();
    /// assert_eq!(client.provider_name(), "Infomaniak");
    /// ```
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let username = config.login.as_ref()
            .ok_or("username is required for Infomaniak")?
            .clone();
        let password = config.password.as_ref()
            .ok_or("password is required for Infomaniak")?
            .clone();
        
        let server = config.server.as_ref()
            .cloned()
            .unwrap_or_else(|| "https://infomaniak.com".to_string());

        Ok(InfomaniakClient {
            server,
            username,
            password,
        })
    }
}

impl DnsClient for InfomaniakClient {
    /// Update the DNS record for a hostname at Infomaniak using the DynDNS2-style update endpoint.
    ///
    /// Sends a GET request to the provider's /nic/update endpoint with Basic authentication and
    /// interprets DynDNS2-style responses to determine success or a specific failure.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the provider reports the update as successful (`good` or `nochg`); `Err` with a
    /// descriptive message for HTTP errors or any provider-reported failure (e.g., authentication
    /// failure, invalid hostname, nonexistent host, abuse, server error, or an unexpected response).
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::net::IpAddr;
    /// # use std::str::FromStr;
    /// # // `client` would be an instance of InfomaniakClient constructed elsewhere.
    /// # fn example_call(client: &crate::clients::infomaniak::InfomaniakClient) {
    /// let hostname = "host.example.com";
    /// let ip = IpAddr::from_str("1.2.3.4").unwrap();
    /// client.update_record(hostname, ip).unwrap();
    /// # }
    /// ```
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let url = format!(
            "{}/nic/update?hostname={}&myip={}",
            self.server, hostname, ip
        );

        log::info!("Updating {} with Infomaniak", hostname);

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

    /// Ensures the client has both username and password configured.
    ///
    /// Returns `Ok(())` if both `username` and `password` are non-empty, `Err` with a descriptive message otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = InfomaniakClient { server: "https://infomaniak.com".into(), username: "user".into(), password: "pass".into() };
    /// client.validate_config().unwrap();
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.username.is_empty() {
            return Err("username is required for Infomaniak".into());
        }
        if self.password.is_empty() {
            return Err("password is required for Infomaniak".into());
        }
        Ok(())
    }

    /// Gets the DNS provider's canonical name.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// // Construct a client (example assumes `Config` and `InfomaniakClient::new` are available).
    /// let config = /* obtain or build a Config */ unimplemented!();
    /// let client = crate::clients::InfomaniakClient::new(&config).unwrap();
    /// assert_eq!(client.provider_name(), "Infomaniak");
    /// ```
    fn provider_name(&self) -> &str {
        "Infomaniak"
    }
}