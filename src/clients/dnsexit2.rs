use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

pub struct Dnsexit2Client {
    api_key: String,
    server: String,
    path: String,
    ttl: u32,
    zone: String,
}

impl Dnsexit2Client {
    /// Create a DNSExit2 client from the given configuration.
    ///
    /// The returned client is configured using `config.password` as the API key,
    /// `config.server` (defaulting to "api.dnsexit.com" when absent), a fixed path
    /// of "/dns/", `config.ttl` (defaulting to 5 when absent), and `config.zone`
    /// (empty string when absent).
    ///
    /// # Returns
    ///
    /// `Ok(Dnsexit2Client)` initialized from `config`; `Err` if the configuration
    /// does not include an API key (`password`).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// // Construct a Config with at least a password, then create the client.
    /// let cfg = Config {
    ///     password: Some("my_api_key".to_string()),
    ///     server: None,
    ///     ttl: None,
    ///     zone: None,
    ///     // other fields...
    /// };
    /// let client = Dnsexit2Client::new(&cfg).expect("failed to create client");
    /// ```
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let api_key = config.password.as_ref()
            .ok_or("API key (password) is required for DNSExit2")?;
        let server = config.server.as_deref()
            .unwrap_or("api.dnsexit.com");
        let path = "/dns/";
        let ttl = config.ttl.unwrap_or(5);
        
        // Zone from zone_id, will be set from hostname if not specified
        let zone = config.zone.clone().unwrap_or_default();

        Ok(Dnsexit2Client {
            api_key: api_key.to_string(),
            server: server.to_string(),
            path: path.to_string(),
            ttl,
            zone,
        })
    }
}

impl DnsClient for Dnsexit2Client {
    /// Updates the DNS record for a hostname at DNSExit2 using the configured client.
    ///
    /// Sends a POST request to the DNSExit2 API to set an A or AAAA record for `hostname` to `ip`.
    /// The client's `zone` is used as the domain; if `zone` is empty, `hostname` is used as the domain.
    /// On success the function returns without error; on failure it returns an error describing the HTTP or API-level problem.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::IpAddr;
    ///
    /// let client = Dnsexit2Client {
    ///     api_key: "secret".into(),
    ///     server: "api.dnsexit.com".into(),
    ///     path: "/dns/".into(),
    ///     ttl: 5,
    ///     zone: "".into(),
    /// };
    ///
    /// // Update an IPv4 record (may perform network I/O in real use)
    /// let _ = client.update_record("host.example.com", "1.2.3.4".parse::<IpAddr>().unwrap());
    /// ```
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        log::info!("Updating DNSExit2 record for {} to {}", hostname, ip);

        // Zone defaults to hostname if not configured
        let zone = if self.zone.is_empty() {
            hostname
        } else {
            &self.zone
        };

        // Determine record type and extract hostname from host
        let (record_type, name) = match ip {
            IpAddr::V4(_) => ("A", hostname.strip_suffix(&format!("  .{}", zone)).unwrap_or("")),
            IpAddr::V6(_) => ("AAAA", hostname.strip_suffix(&format!(".{}", zone)).unwrap_or("")),
        };

        // Build JSON payload
        let json_payload = format!(
            r#"{{"apikey":"{}","domain":"{}","update":[{{"type":"{}","name":"{}","content":"{}","ttl":{}}}]}}"#,
            self.api_key, zone, record_type, name, ip, self.ttl
        );

        let url = format!("https://{}{}", self.server, self.path);

        let response = minreq::post(&url)
            .with_header("Content-Type", "application/json")
            .with_header("User-Agent", crate::USER_AGENT)
            .with_body(json_payload)
            .send()?;

        if response.status_code != 200 {
            return Err(format!("DNSExit2 API error: HTTP {}", response.status_code).into());
        }

        let body = response.as_str()?;
        
        if body.contains("\"code\":0") || body.contains("\"message\":\"Success\"") {
            log::info!("Successfully updated DNS record for {} to {}", hostname, ip);
            Ok(())
        } else if body.contains("\"code\":") {
            let error_msg = body
                .split("\"message\":\"")
                .nth(1)
                .and_then(|s| s.split("\"").next())
                .unwrap_or("Unknown error");
            Err(format!("DNSExit2 error: {}", error_msg).into())
        } else {
            Err(format!("Unexpected DNSExit2 response: {}", body).into())
        }
    }

    /// Validates the client's configuration by ensuring an API key is set.
    ///
    /// Returns `Ok(())` if the API key is not empty, `Err` with a descriptive message otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = Dnsexit2Client {
    ///     api_key: "secret".to_string(),
    ///     server: "api.dnsexit.com".to_string(),
    ///     path: "/dns/".to_string(),
    ///     ttl: 5,
    ///     zone: "".to_string(),
    /// };
    /// assert!(client.validate_config().is_ok());
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.api_key.is_empty() {
            return Err("DNSExit2 API key cannot be empty".into());
        }
        Ok(())
    }

    /// Provides the provider identifier for this client.
    ///
    /// # Returns
    ///
    /// The static provider name "DNSExit2".
    ///
    /// # Examples
    ///
    /// ```
    /// let client = Dnsexit2Client {
    ///     api_key: String::new(),
    ///     server: String::new(),
    ///     path: String::new(),
    ///     ttl: 5,
    ///     zone: String::new(),
    /// };
    /// assert_eq!(client.provider_name(), "DNSExit2");
    /// ```
    fn provider_name(&self) -> &'static str {
        "DNSExit2"
    }
}