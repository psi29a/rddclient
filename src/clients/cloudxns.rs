use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

/// CloudXNS DNS client
/// Uses CloudXNS REST API
pub struct CloudXnsClient {
    server: String,
    api_key: String,
    secret_key: String,
}

impl CloudXnsClient {
    /// Creates a CloudXnsClient from a Config by extracting CloudXNS credentials and server URL.
    ///
    /// The `login` field of `config` is used as the API key and must be present. The `password` field
    /// is used as the secret key and must be present. If `server` is not provided, the default
    /// "https://www.cloudxns.net" is used.
    ///
    /// # Errors
    ///
    /// Returns an error if `login` (API key) or `password` (secret key) is missing in `config`.
    ///
    /// # Examples
    ///
    /// ```
    /// let cfg = Config {
    ///     login: Some("api_key".into()),
    ///     password: Some("secret_key".into()),
    ///     server: None,
    ///     ..Default::default()
    /// };
    /// let client = CloudXnsClient::new(&cfg).unwrap();
    /// assert_eq!(client.provider_name(), "CloudXNS");
    /// ```
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let api_key = config.login.as_ref()
            .ok_or("username (API key) is required for CloudXNS")?
            .clone();
        let secret_key = config.password.as_ref()
            .ok_or("password (secret key) is required for CloudXNS")?
            .clone();
        
        let server = config.server.clone()
            .unwrap_or_else(|| "https://www.cloudxns.net".to_string());

        Ok(CloudXnsClient {
            server,
            api_key,
            secret_key,
        })
    }
}

impl DnsClient for CloudXnsClient {
    /// Update the DNS record for a hostname on CloudXNS to the provided IP address.
    ///
    /// Attempts to push an "A" record for IPv4 or "AAAA" record for IPv6 to the CloudXNS DDNS endpoint.
    /// The result is considered successful when CloudXNS returns a success indicator in the response body.
    ///
    /// # Returns
    ///
    /// `Ok(())` if CloudXNS accepted the update; `Err` with a descriptive message otherwise.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use std::net::IpAddr;
    ///
    /// let client = CloudXnsClient {
    ///     server: "https://www.cloudxns.net".to_string(),
    ///     api_key: "APIKEY".to_string(),
    ///     secret_key: "SECRET".to_string(),
    /// };
    ///
    /// let ip: IpAddr = "203.0.113.42".parse().unwrap();
    /// client.update_record("host.example.com", ip).expect("update failed");
    /// ```
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let record_type = match ip {
            IpAddr::V4(_) => "A",
            IpAddr::V6(_) => "AAAA",
        };

        // CloudXNS API endpoint
        let url = format!("{}/api2/ddns", self.server);

        log::info!("Updating {} with CloudXNS", hostname);

        // CloudXNS uses custom authentication headers
        let response = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .with_header("API-KEY", &self.api_key)
            .with_header("API-REQUEST-DATE", format!("{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()))
            .with_param("domain", hostname)
            .with_param("ip", ip.to_string())
            .with_param("type", record_type)
            .send()?;

        let status_code = response.status_code;
        let body = response.as_str()?.trim();

        log::debug!("Response status: {}, body: {}", status_code, body);

        if status_code != 200 {
            return Err(format!("HTTP error: {}", status_code).into());
        }

        // Parse JSON response
        if body.contains(r#""code":1"#) || body.contains("success") {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else if body.contains("error") {
            Err(format!("CloudXNS error: {}", body).into())
        } else {
            Err(format!("Unexpected response: {}", body).into())
        }
    }

    /// Ensure the client has both an API key and a secret key configured.
    ///
    /// Returns `Ok(())` if `api_key` and `secret_key` are both non-empty; returns an `Err` describing
    /// which credential is missing otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = CloudXnsClient {
    ///     server: "https://www.cloudxns.net".to_string(),
    ///     api_key: "my-api-key".to_string(),
    ///     secret_key: "my-secret".to_string(),
    /// };
    /// assert!(client.validate_config().is_ok());
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.api_key.is_empty() {
            return Err("username (API key) is required for CloudXNS".into());
        }
        if self.secret_key.is_empty() {
            return Err("password (secret key) is required for CloudXNS".into());
        }
        Ok(())
    }

    /// The DNS provider identifier returned by this client.
    ///
    /// # Examples
    ///
    /// ```
    /// // Construct a client (replace `config` with a valid Config instance)
    /// let config = /* Config with valid api_key and secret_key */;
    /// let client = CloudXnsClient::new(&config).unwrap();
    /// assert_eq!(client.provider_name(), "CloudXNS");
    /// ```
    fn provider_name(&self) -> &str {
        "CloudXNS"
    }
}