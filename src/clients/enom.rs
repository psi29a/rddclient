use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

/// Enom DNS client
/// Uses Enom's Dynamic DNS API
pub struct EnomClient {
    server: String,
    password: String,
}

impl EnomClient {
    /// Creates an EnomClient from the given configuration.
    ///
    /// Returns an error if `config.password` is missing. When `config.server` is not set,
    /// the default "https://dynamic.name-services.com" is used for the Enom API server.
    ///
    /// # Examples
    ///
    /// ```
    /// let cfg = Config {
    ///     password: Some("update-token".to_string()),
    ///     server: None,
    ///     // other fields...
    /// };
    /// let client = EnomClient::new(&cfg).unwrap();
    /// assert_eq!(client.provider_name(), "Enom");
    /// ```
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let password = config.password.as_ref()
            .ok_or("password (update token) is required for Enom")?
            .clone();
        
        let server = config.server.as_ref()
            .cloned()
            .unwrap_or_else(|| "https://dynamic.name-services.com".to_string());

        Ok(EnomClient {
            server,
            password,
        })
    }
}

impl DnsClient for EnomClient {
    /// Update the DNS record for a hostname on Enom's Dynamic DNS service.
    ///
    /// Builds a SetDNSHost request using the hostname (subdomain portion as `HostName`, remainder as `Zone`),
    /// the provided IP address, and the client's domain password, then issues an HTTP GET to the Enom API.
    /// The call is considered successful when the HTTP status is 200 and the response body indicates success
    /// (e.g., contains `ErrCount=0` or `success`). Errors are returned for non-200 responses or when the
    /// Enom response contains an error indicator or is otherwise unexpected.
    ///
    /// # Returns
    ///
    /// `Ok(())` on successful update; `Err` containing a descriptive message when the HTTP request fails,
    /// the status is not 200, or the Enom response indicates an error.
    ///
    /// # Examples
    ///
    /// ```
    /// // assuming `client` is an initialized EnomClient and `ip` is an IpAddr
    /// let _ = client.update_record("host.example.com", ip);
    /// ```
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let url = format!(
            "{}/interface.asp?command=SetDNSHost&HostName={}&Zone={}&Address={}&DomainPassword={}",
            self.server,
            hostname.split('.').next().unwrap_or(""),
            hostname.split('.').skip(1).collect::<Vec<_>>().join("."),
            ip,
            self.password
        );

        log::info!("Updating {} with Enom", hostname);

        let response = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .send()?;

        let status_code = response.status_code;
        let body = response.as_str()?.trim();

        log::debug!("Response status: {}, body: {}", status_code, body);

        if status_code != 200 {
            return Err(format!("HTTP error: {}", status_code).into());
        }

        // Parse Enom response
        if body.contains("ErrCount=0") || body.contains("success") {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else if body.contains("ErrCount=") {
            Err(format!("Enom error: {}", body).into())
        } else {
            Err(format!("Unexpected response: {}", body).into())
        }
    }

    /// Ensure the client has a non-empty Enom update password.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the client's password is present, `Err` with a message otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = EnomClient { server: "https://dynamic.name-services.com".into(), password: "token".into() };
    /// assert!(client.validate_config().is_ok());
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.password.is_empty() {
            return Err("password (update token) is required for Enom".into());
        }
        Ok(())
    }

    /// DNS provider identifier for this client.
    ///
    /// # Returns
    ///
    /// The literal string `"Enom"`.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = crate::clients::enom::EnomClient { server: String::new(), password: String::new() };
    /// assert_eq!(client.provider_name(), "Enom");
    /// ```
    fn provider_name(&self) -> &str {
        "Enom"
    }
}