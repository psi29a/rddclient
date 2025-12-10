use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

/// LuaDNS client
/// Uses LuaDNS REST API
pub struct LuadnsClient {
    server: String,
    email: String,
    token: String,
    zone_id: String,
    record_id: String,
}

impl LuadnsClient {
    /// Creates a new `LuadnsClient` from the provided `Config`, validating required fields and using a default API server if none is specified.
    ///
    /// The function extracts `login` (email), `password` (API token), `zone`, and `host` (record ID) from `config` and returns an error if any of these are missing. If `config.server` is `None`, the server defaults to `https://api.luadns.com`.
    ///
    /// # Returns
    ///
    /// `Ok(LuadnsClient)` when all required configuration fields are present, or `Err` with a descriptive message when a required field is missing.
    ///
    /// # Examples
    ///
    /// ```
    /// // Construct a minimal Config with required fields.
    /// let config = Config {
    ///     login: Some("user@example.com".to_string()),
    ///     password: Some("secret-token".to_string()),
    ///     zone: Some("zone-id".to_string()),
    ///     host: Some("record-id".to_string()),
    ///     server: None,
    ///     ..Default::default()
    /// };
    ///
    /// let client = LuadnsClient::new(&config).expect("valid config");
    /// assert_eq!(client.provider_name(), "LuaDNS");
    /// ```
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let email = config.login.as_ref()
            .ok_or("username (email) is required for LuaDNS")?
            .clone();
        
        let token = config.password.as_ref()
            .ok_or("api_token is required for LuaDNS")?
            .clone();
        
        let zone_id = config.zone.as_ref()
            .ok_or("zone_id is required for LuaDNS")?
            .clone();
        
        let record_id = config.host.as_ref()
            .ok_or("dns_record (record ID) is required for LuaDNS")?
            .clone();
        
        let server = config.server.clone()
            .unwrap_or_else(|| "https://api.luadns.com".to_string());

        Ok(LuadnsClient {
            server,
            email,
            token,
            zone_id,
            record_id,
        })
    }
}

impl DnsClient for LuadnsClient {
    /// Update the DNS record for a hostname to the provided IP address using the LuaDNS REST API.
    ///
    /// On success the record is updated on the remote provider and the method returns `Ok(())`.
    /// If the API responds with an error payload the function returns `Err` containing the API body;
    /// for other non-200 HTTP responses it returns `Err` containing the numeric status code.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::IpAddr;
    ///
    /// let client = LuadnsClient {
    ///     server: "https://api.luadns.com".into(),
    ///     email: "user@example.com".into(),
    ///     token: "token".into(),
    ///     zone_id: "zone".into(),
    ///     record_id: "record".into(),
    /// };
    ///
    /// // IPv4 example
    /// client.update_record("host.example.com", "1.2.3.4".parse::<IpAddr>().unwrap()).unwrap();
    /// ```
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let record_type = match ip {
            IpAddr::V4(_) => "A",
            IpAddr::V6(_) => "AAAA",
        };

        log::info!("Updating {} with LuaDNS", hostname);

        // LuaDNS API endpoint
        let url = format!("{}/v1/zones/{}/records/{}", 
            self.server, self.zone_id, self.record_id);

        let body = format!(
            r#"{{"content":"{}","type":"{}"}}"#,
            ip,
            record_type
        );

        let auth = format!("{}:{}", self.email, self.token);
        let encoded_auth = format!("Basic {}", base64::encode(&auth));

        let response = minreq::put(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .with_header("Authorization", &encoded_auth)
            .with_header("Content-Type", "application/json")
            .with_header("Accept", "application/json")
            .with_body(body)
            .send()?;

        let status_code = response.status_code;
        let response_body = response.as_str()?.trim();

        log::debug!("Response status: {}, body: {}", status_code, response_body);

        if status_code == 200 {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else if response_body.contains("error") {
            Err(format!("LuaDNS API error: {}", response_body).into())
        } else {
            Err(format!("HTTP error: {}", status_code).into())
        }
    }

    /// Validates that required LuaDNS configuration fields are present.
    ///
    /// Checks that `email`, `token`, `zone_id`, and `record_id` are not empty.
    ///
    /// # Returns
    ///
    /// `Ok(())` if all required fields are non-empty; `Err` with a descriptive message if any field is missing.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = LuadnsClient {
    ///     server: "https://api.luadns.com".into(),
    ///     email: "user@example.com".into(),
    ///     token: "secret".into(),
    ///     zone_id: "zone123".into(),
    ///     record_id: "rec456".into(),
    /// };
    /// assert!(client.validate_config().is_ok());
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.email.is_empty() {
            return Err("username (email) is required for LuaDNS".into());
        }
        if self.token.is_empty() {
            return Err("api_token is required for LuaDNS".into());
        }
        if self.zone_id.is_empty() {
            return Err("zone_id is required for LuaDNS".into());
        }
        if self.record_id.is_empty() {
            return Err("dns_record (record ID) is required for LuaDNS".into());
        }
        Ok(())
    }

    /// Get the DNS provider name for this client.
    ///
    /// # Returns
    ///
    /// The provider name `"LuaDNS"`.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = LuadnsClient {
    ///     server: String::new(),
    ///     email: String::new(),
    ///     token: String::new(),
    ///     zone_id: String::new(),
    ///     record_id: String::new(),
    /// };
    /// assert_eq!(client.provider_name(), "LuaDNS");
    /// ```
    fn provider_name(&self) -> &str {
        "LuaDNS"
    }
}

mod base64 {
    use base64::{Engine as _, engine::general_purpose};
    
    /// Encodes the given string into Base64 using the standard encoding.
    ///
    /// # Examples
    ///
    /// ```
    /// let out = encode("hello");
    /// assert_eq!(out, "aGVsbG8=");
    /// ```
    pub fn encode(data: &str) -> String {
        general_purpose::STANDARD.encode(data.as_bytes())
    }
}