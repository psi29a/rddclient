use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

/// DonDominio DNS client
/// Uses DonDominio's dondns API with API key authentication
pub struct DonDominioClient {
    server: String,
    api_key: String,
    username: String,
}

impl DonDominioClient {
    /// Constructs a DonDominioClient from a Config by extracting the required credentials and optional server.
    ///
    /// Returns an error if the API key (config.password) or username (config.login) is missing:
    /// - Error message "password (API key) is required for DonDominio" when `config.password` is absent.
    /// - Error message "username is required for DonDominio" when `config.login` is absent.
    /// The server URL defaults to "https://dondns.dondominio.com" when `config.server` is not provided.
    ///
    /// # Examples
    ///
    /// ```
    /// // Assuming `Config` has fields: `login: Option<String>`, `password: Option<String>`, `server: Option<String>`
    /// let cfg = Config {
    ///     login: Some("user@example.com".to_string()),
    ///     password: Some("secret_api_key".to_string()),
    ///     server: None,
    /// };
    /// let client = DonDominioClient::new(&cfg).unwrap();
    /// assert_eq!(client.provider_name(), "DonDominio");
    /// ```
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let api_key = config.password.as_ref()
            .ok_or("password (API key) is required for DonDominio")?
            .clone();
        let username = config.login.as_ref()
            .ok_or("username is required for DonDominio")?
            .clone();
        
        let server = config.server.as_ref()
            .cloned()
            .unwrap_or_else(|| "https://dondns.dondominio.com".to_string());

        Ok(DonDominioClient {
            server,
            api_key,
            username,
        })
    }
}

impl DnsClient for DonDominioClient {
    /// Update the DNS record for a hostname to the given IP on DonDominio.
    ///
    /// Attempts to update the record at the provider and returns success only when
    /// the provider acknowledges the change.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the update is acknowledged by DonDominio. `Err` when the HTTP
    /// request fails, the provider responds with a non-200 status code, the response
    /// indicates authentication failure, the provider reports an error, or the
    /// response is not recognised.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::IpAddr;
    /// // `client` must be an initialized DonDominioClient (shown here as a placeholder).
    /// let client: DonDominioClient = /* initialized client */;
    /// let _ = client.update_record("host.example.com", "1.2.3.4".parse::<IpAddr>().unwrap());
    /// ```
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let record_type = match ip {
            IpAddr::V4(_) => "A",
            IpAddr::V6(_) => "AAAA",
        };

        let url = format!("{}/update", self.server);
        
        let json_body = format!(
            r#"{{"apiuser":"{}","apipasswd":"{}","domain":"{}","name":"{}","type":"{}","value":"{}"}}"#,
            self.username,
            self.api_key,
            hostname.split('.').skip(1).collect::<Vec<_>>().join("."),
            hostname.split('.').next().unwrap_or(""),
            record_type,
            ip
        );

        log::info!("Updating {} with DonDominio", hostname);

        let response = minreq::post(&url)
            .with_header("Content-Type", "application/json")
            .with_header("User-Agent", crate::USER_AGENT)
            .with_body(json_body)
            .send()?;

        let status_code = response.status_code;
        let body = response.as_str()?.trim();

        log::debug!("Response status: {}, body: {}", status_code, body);

        if status_code != 200 {
            return Err(format!("HTTP error: {}", status_code).into());
        }

        // Parse JSON response
        if body.contains(r#""success":true"#) || body.contains(r#""success":"true""#) {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else if body.contains("authentication") || body.contains("credentials") {
            Err("Authentication failed - check username and API key".into())
        } else if body.contains("error") {
            Err(format!("DonDominio error: {}", body).into())
        } else {
            Err(format!("Unexpected response: {}", body).into())
        }
    }

    /// Ensures the DonDominio client has both a username and an API key configured.
    ///
    /// Returns `Ok(())` when `username` and `api_key` are non-empty. Returns an `Err` with a descriptive message when either value is missing.
    ///
    /// # Examples
    ///
    /// ```
    /// let good = DonDominioClient {
    ///     server: "https://dondns.dondominio.com".into(),
    ///     api_key: "secret".into(),
    ///     username: "user".into(),
    /// };
    /// assert!(good.validate_config().is_ok());
    ///
    /// let missing_key = DonDominioClient {
    ///     server: "https://dondns.dondominio.com".into(),
    ///     api_key: "".into(),
    ///     username: "user".into(),
    /// };
    /// assert!(missing_key.validate_config().is_err());
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.username.is_empty() {
            return Err("username is required for DonDominio".into());
        }
        if self.api_key.is_empty() {
            return Err("password (API key) is required for DonDominio".into());
        }
        Ok(())
    }

    /// Provides the DNS provider name for this client.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = DonDominioClient { server: String::from("https://dondns.dondominio.com"), api_key: String::from("key"), username: String::from("user") };
    /// assert_eq!(client.provider_name(), "DonDominio");
    /// ```
    fn provider_name(&self) -> &str {
        "DonDominio"
    }
}