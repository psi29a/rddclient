use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

/// Linode DNS client
/// Uses Linode API v4
pub struct LinodeClient {
    server: String,
    token: String,
    domain_id: String,
    record_id: String,
}

impl LinodeClient {
    /// Construct a LinodeClient from a Config, validating required fields and applying defaults.
    ///
    /// Returns an error if the configuration is missing the API token, domain ID, or DNS record ID.
    /// If `server` is not provided in the config, the Linode API base URL `https://api.linode.com` is used.
    ///
    /// # Errors
    ///
    /// Returns an error with the message `"api_token is required for Linode"` if the API token is missing,
    /// `"zone_id (domain ID) is required for Linode"` if the domain ID is missing, or
    /// `"dns_record (record ID) is required for Linode"` if the DNS record ID is missing.
    ///
    /// # Examples
    ///
    /// ```
    /// let cfg = Config {
    ///     password: Some("token".to_string()),
    ///     zone: Some("123".to_string()),
    ///     host: Some("456".to_string()),
    ///     server: None,
    ///     ..Default::default()
    /// };
    /// let client = LinodeClient::new(&cfg).unwrap();
    /// assert_eq!(client.provider_name(), "Linode");
    /// ```
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let token = config.password.as_ref()
            .ok_or("api_token is required for Linode")?
            .clone();
        
        let domain_id = config.zone.as_ref()
            .ok_or("zone_id (domain ID) is required for Linode")?
            .clone();
        
        let record_id = config.host.as_ref()
            .ok_or("dns_record (record ID) is required for Linode")?
            .clone();
        
        let server = config.server.clone()
            .unwrap_or_else(|| "https://api.linode.com".to_string());

        Ok(LinodeClient {
            server,
            token,
            domain_id,
            record_id,
        })
    }
}

impl DnsClient for LinodeClient {
    /// Update the DNS record for `hostname` to the given `ip` using the Linode API v4.
    ///
    /// Sends an authenticated HTTP PUT to "{server}/v4/domains/{domain_id}/records/{record_id}"
    /// with a JSON body containing the record `type` ("A" for IPv4, "AAAA" for IPv6) and `target` (the IP).
    ///
    /// Returns `Ok(())` if Linode responds with HTTP 200; returns `Err` containing the API response
    /// when Linode reports errors or when the HTTP status is not 200. Network and serialization
    /// errors from the request are propagated as `Err`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::IpAddr;
    ///
    /// let client = LinodeClient {
    ///     server: "https://api.linode.com".into(),
    ///     token: "MY_TOKEN".into(),
    ///     domain_id: "12345".into(),
    ///     record_id: "67890".into(),
    /// };
    ///
    /// let ip: IpAddr = "203.0.113.1".parse().unwrap();
    /// client.update_record("example.com", ip).unwrap();
    /// ```
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let record_type = match ip {
            IpAddr::V4(_) => "A",
            IpAddr::V6(_) => "AAAA",
        };

        log::info!("Updating {} with Linode", hostname);

        // Linode API v4 endpoint
        let url = format!("{}/v4/domains/{}/records/{}", 
            self.server, self.domain_id, self.record_id);

        let body = format!(
            r#"{{"type":"{}","target":"{}"}}"#,
            record_type,
            ip
        );

        let response = minreq::put(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .with_header("Authorization", format!("Bearer {}", self.token))
            .with_header("Content-Type", "application/json")
            .with_body(body)
            .send()?;

        let status_code = response.status_code;
        let response_body = response.as_str()?.trim();

        log::debug!("Response status: {}, body: {}", status_code, response_body);

        if status_code == 200 {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else if response_body.contains("errors") {
            Err(format!("Linode API error: {}", response_body).into())
        } else {
            Err(format!("HTTP error: {}", status_code).into())
        }
    }

    /// Validate that the Linode client has all required configuration fields set.
    ///
    /// Returns an error with a descriptive message when the API token, domain ID, or
    /// DNS record ID is missing; returns `Ok(())` when all required fields are present.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = LinodeClient {
    ///     server: "https://api.linode.com".into(),
    ///     token: "".into(),
    ///     domain_id: "".into(),
    ///     record_id: "".into(),
    /// };
    /// assert!(client.validate_config().is_err());
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.token.is_empty() {
            return Err("api_token is required for Linode".into());
        }
        if self.domain_id.is_empty() {
            return Err("zone_id (domain ID) is required for Linode".into());
        }
        if self.record_id.is_empty() {
            return Err("dns_record (record ID) is required for Linode".into());
        }
        Ok(())
    }

    /// DNS provider name for this client.
    ///
    /// # Returns
    ///
    /// `"Linode"` — the provider identifier.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = LinodeClient { server: String::new(), token: String::new(), domain_id: String::new(), record_id: String::new() };
    /// assert_eq!(client.provider_name(), "Linode");
    /// ```
    fn provider_name(&self) -> &str {
        "Linode"
    }
}