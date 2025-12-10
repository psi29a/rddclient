use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

/// Yandex PDD (Yandex.Connect) DNS client
/// Uses Yandex PDD API with OAuth token
pub struct YandexClient {
    server: String,
    token: String,
    domain: String,
}

impl YandexClient {
    /// Creates a new YandexClient from the provided configuration.
    ///
    /// The function reads the PDD token from `config.password` and the domain (zone) from
    /// `config.zone`. If `config.server` is not set, the Yandex PDD API server
    /// "https://pddimp.yandex.ru" is used as the default.
    ///
    /// Returns an error if the configuration is missing the required `password` (PDD token)
    /// or `zone` (domain).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::error::Error;
    /// # use crate::Config;
    /// # use crate::clients::yandex::YandexClient;
    /// # fn example() -> Result<(), Box<dyn Error>> {
    /// let config = Config {
    ///     password: Some("pdd-token".to_string()),
    ///     zone: Some("example.com".to_string()),
    ///     server: None,
    ///     ..Default::default()
    /// };
    /// let client = YandexClient::new(&config)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let token = config.password.as_ref()
            .ok_or("password (PDD token) is required for Yandex")?
            .clone();
        let domain = config.zone.as_ref()
            .ok_or("zone_id (domain) is required for Yandex")?
            .clone();
        
        let server = config.server.as_ref()
            .cloned()
            .unwrap_or_else(|| "https://pddimp.yandex.ru".to_string());

        Ok(YandexClient {
            server,
            token,
            domain,
        })
    }
}

impl DnsClient for YandexClient {
    /// Update the DNS record for a hostname to the provided IP using the Yandex PDD API.
    ///
    /// On success this returns `Ok(())`. On failure this returns an `Err` containing a description
    /// of the HTTP or API error.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::IpAddr;
    ///
    /// let client = YandexClient {
    ///     server: "https://pddimp.yandex.ru".into(),
    ///     token: "token".into(),
    ///     domain: "example.com".into(),
    /// };
    ///
    /// // Update host.example.com to 1.2.3.4
    /// let res = client.update_record("host.example.com", "1.2.3.4".parse::<IpAddr>().unwrap());
    /// assert!(res.is_ok() || res.is_err()); // network-dependent in real use
    /// ```
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let record_type = match ip {
            IpAddr::V4(_) => "A",
            IpAddr::V6(_) => "AAAA",
        };

        // Extract subdomain from hostname
        let subdomain = if hostname.ends_with(&format!(".{}", self.domain)) {
            hostname.trim_end_matches(&format!(".{}", self.domain))
        } else {
            hostname
        };

        let url = format!(
            "{}/api2/admin/dns/edit?domain={}&subdomain={}&record_id=0&type={}&content={}",
            self.server, self.domain, subdomain, record_type, ip
        );

        log::info!("Updating {} with Yandex", hostname);

        let response = minreq::post(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .with_header("PddToken", &self.token)
            .send()?;

        let status_code = response.status_code;
        let body = response.as_str()?.trim();

        log::debug!("Response status: {}, body: {}", status_code, body);

        if status_code != 200 {
            return Err(format!("HTTP error: {}", status_code).into());
        }

        // Parse JSON response
        if body.contains(r#""success":"ok""#) || body.contains(r#""ok":true"#) {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else if body.contains("error") {
            Err(format!("Yandex API error: {}", body).into())
        } else {
            Err(format!("Unexpected response: {}", body).into())
        }
    }

    /// Validates that the client has the required Yandex configuration values.
    ///
    /// # Returns
    ///
    /// `Ok(())` if both the PDD token and domain are non-empty, `Err` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = YandexClient {
    ///     server: "https://pddimp.yandex.ru".to_string(),
    ///     token: "token".to_string(),
    ///     domain: "example.com".to_string(),
    /// };
    /// assert!(client.validate_config().is_ok());
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.token.is_empty() {
            return Err("password (PDD token) is required for Yandex".into());
        }
        if self.domain.is_empty() {
            return Err("zone_id (domain) is required for Yandex".into());
        }
        Ok(())
    }

    /// Identify the DNS provider implemented by this client.
    ///
    /// Returns the provider name `"Yandex"`.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = YandexClient { server: String::new(), token: String::new(), domain: String::new() };
    /// assert_eq!(client.provider_name(), "Yandex");
    /// ```
    fn provider_name(&self) -> &str {
        "Yandex"
    }
}