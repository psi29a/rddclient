use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;
use base64::{Engine as _, engine::general_purpose};

/// Zoneedit client - DynDNS2-compatible
pub struct ZoneeditClient {
    username: String,
    password: String,
    server: String,
}

impl ZoneeditClient {
    /// Creates a ZoneeditClient from configuration, applying a default server URL when absent.
    ///
    /// Validates that `config.login` and `config.password` are present and uses `config.server` if provided,
    /// otherwise defaults to "https://dynamic.zoneedit.com".
    ///
    /// # Returns
    ///
    /// `Ok(ZoneeditClient)` when both username and password are present in `config`, `Err` with a message if either is missing.
    ///
    /// # Examples
    ///
    /// ```
    /// # use your_crate::{Config, ZoneeditClient, DnsClient};
    /// let cfg = Config {
    ///     login: Some("user".into()),
    ///     password: Some("pass".into()),
    ///     server: None,
    ///     ..Default::default()
    /// };
    /// let client = ZoneeditClient::new(&cfg).unwrap();
    /// assert_eq!(client.provider_name(), "Zoneedit");
    /// ```
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let username = config.login.as_ref()
            .ok_or("username is required for Zoneedit")?
            .clone();
        let password = config.password.as_ref()
            .ok_or("password is required for Zoneedit")?
            .clone();
        
        let server = config.server.as_ref()
            .cloned()
            .unwrap_or_else(|| "https://dynamic.zoneedit.com".to_string());

        Ok(ZoneeditClient {
            username,
            password,
            server,
        })
    }
}

impl DnsClient for ZoneeditClient {
    /// Update the DNS A record for the given hostname on Zoneedit.
    ///
    /// Sends a dynamic update request to the configured Zoneedit server using the client's
    /// credentials and interprets the provider response to determine success or failure.
    ///
    /// # Parameters
    ///
    /// - `hostname`: the DNS host name to update (for example `host.example.com`).
    /// - `ip`: the IP address to assign to the host.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the provider response contains `<SUCCESS>`. `Err` with a boxed error
    /// describing the failure if the response contains `<ERROR>` or any other unexpected content.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// // given a configured `client: ZoneeditClient`
    /// client.update_record("host.example.com", "1.2.3.4".parse().unwrap()).unwrap();
    /// ```
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let url = format!(
            "{}/auth/dynamic.html?host={}&dnsto={}",
            self.server, hostname, ip
        );

        log::info!("Updating {} with Zoneedit", hostname);

        let response = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .with_header("Authorization", format!("Basic {}", 
                general_purpose::STANDARD.encode(format!("{}:{}", self.username, self.password))))
            .send()?;

        let body = response.as_str()?;

        if body.contains("<SUCCESS") {
            log::info!("DNS record for {} successfully updated to {}", hostname, ip);
            Ok(())
        } else if body.contains("<ERROR") {
            Err("Zoneedit update failed - check credentials and hostname".into())
        } else {
            Err(format!("Unexpected response: {}", body).into())
        }
    }

    /// Validates that the client has both a username and password set for Zoneedit.
    ///
    /// Returns an error if the username or password is empty.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// // assume `client` is a configured ZoneeditClient
    /// let result = client.validate_config();
    /// assert!(result.is_ok());
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.username.is_empty() {
            return Err("username is required for Zoneedit".into());
        }
        if self.password.is_empty() {
            return Err("password is required for Zoneedit".into());
        }
        Ok(())
    }

    /// Provider name for this client.
    ///
    /// # Returns
    ///
    /// `&str` with the provider name "Zoneedit".
    ///
    /// # Examples
    ///
    /// ```no_run
    /// // given a `ZoneeditClient` instance `client`:
    /// // assert_eq!(client.provider_name(), "Zoneedit");
    /// ```
    fn provider_name(&self) -> &str {
        "Zoneedit"
    }
}