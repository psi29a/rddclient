use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;
use serde_json::json;

/// OVH client - https://api.ovh.com/
/// Note: OVH requires application key/secret and consumer key
pub struct OvhClient {
    application_key: String,
    application_secret: String,
    consumer_key: String,
    server: String,
}

impl OvhClient {
    /// Creates an `OvhClient` from a `Config`, extracting the required OVH credentials and server URL.
    ///
    /// This reads the application key from `config.login`, the application secret and consumer key from
    /// `config.password`, and the server URL from `config.server`. If `config.server` is not set,
    /// the default `"https://eu.api.ovh.com/1.0"` is used.
    ///
    /// # Errors
    ///
    /// Returns an error if `login` or `password` are missing; the error message indicates which credential is required.
    ///
    /// # Examples
    ///
    /// ```
    /// let cfg = Config {
    ///     login: Some("app_key".into()),
    ///     password: Some("app_secret".into()),
    ///     server: None,
    ///     ..Default::default()
    /// };
    /// let client = OvhClient::new(&cfg).unwrap();
    /// ```
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        // username = application_key, password = application_secret, api_token = consumer_key
        let application_key = config.login.as_ref()
            .ok_or("username (application key) is required for OVH")?
            .clone();
        let application_secret = config.password.as_ref()
            .ok_or("password (application secret) is required for OVH")?
            .clone();
        let consumer_key = config.password.as_ref()
            .ok_or("api_token (consumer key) is required for OVH")?
            .clone();
        
        let server = config.server.as_ref()
            .cloned()
            .unwrap_or_else(|| "https://eu.api.ovh.com/1.0".to_string());

        Ok(OvhClient {
            application_key,
            application_secret,
            consumer_key,
            server,
        })
    }

    /// Split a hostname into its subdomain and the registered domain (last two labels).
    ///
    /// Returns `(subdomain, domain)`. If the hostname contains at least two dots (three labels or more),
    /// `domain` is the last two labels joined by a dot and `subdomain` is everything to the left of that.
    /// If the hostname contains fewer than two dots, `subdomain` is an empty string and `domain` is the original hostname.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = OvhClient {
    ///     application_key: "".into(),
    ///     application_secret: "".into(),
    ///     consumer_key: "".into(),
    ///     server: "https://eu.api.ovh.com/1.0".into(),
    /// };
    /// let (sub, domain) = client.parse_hostname("www.sub.example.com");
    /// assert_eq!(sub, "www.sub");
    /// assert_eq!(domain, "example.com");
    ///
    /// let (sub, domain) = client.parse_hostname("example.com");
    /// assert_eq!(sub, "");
    /// assert_eq!(domain, "example.com");
    /// ```
    fn parse_hostname(&self, hostname: &str) -> (String, String) {
        let parts: Vec<&str> = hostname.rsplitn(3, '.').collect();
        if parts.len() >= 3 {
            let domain = format!("{}.{}", parts[1], parts[0]);
            let subdomain = parts[2];
            (subdomain.to_string(), domain)
        } else {
            ("".to_string(), hostname.to_string())
        }
    }
}

impl DnsClient for OvhClient {
    /// Updates the DNS record for a hostname on OVH to the provided IP address.
    ///
    /// Sends a request to the OVH domain zone API to create or update an A/AAAA record
    /// for the hostname (record type chosen from the IP version). This implementation
    /// uses a simplified request flow and does not perform OVH request signing; proper
    /// signing is required for production use.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success; `Err` with a descriptive message if the OVH API returns an error.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use std::net::IpAddr;
    ///
    /// let client = OvhClient {
    ///     application_key: "app_key".into(),
    ///     application_secret: "app_secret".into(),
    ///     consumer_key: "consumer_key".into(),
    ///     server: "https://eu.api.ovh.com/1.0".into(),
    /// };
    ///
    /// client.update_record("www.example.com", "1.2.3.4".parse::<IpAddr>().unwrap()).unwrap();
    /// ```
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let (subdomain, domain) = self.parse_hostname(hostname);
        
        // Determine record type based on IP version
        let record_type = match ip {
            IpAddr::V4(_) => "A",
            IpAddr::V6(_) => "AAAA",
        };
        
        // Note: Full OVH implementation requires request signing
        // This is a simplified version - production use requires proper signing
        let url = format!("{}/domain/zone/{}/record", self.server, domain);

        log::info!("Updating {} with OVH (simplified API, {})", hostname, record_type);
        log::warn!("OVH implementation requires proper request signing for production use");

        let body = json!({
            "fieldType": record_type,
            "subDomain": subdomain,
            "target": ip.to_string()
        });

        let response = minreq::post(&url)
            .with_header("X-Ovh-Application", &self.application_key)
            .with_header("X-Ovh-Consumer", &self.consumer_key)
            .with_header("Content-Type", "application/json")
            .with_json(&body)?
            .send()?;

        let status_code = response.status_code;

        if status_code == 200 || status_code == 201 {
            log::info!("DNS record for {} successfully updated to {}", hostname, ip);
            Ok(())
        } else {
            let body = response.as_str().unwrap_or("unknown error");
            Err(format!("OVH API error ({}): {}", status_code, body).into())
        }
    }

    /// Validates that the OVH client has all required credentials configured.
    ///
    /// Checks that `application_key`, `application_secret`, and `consumer_key` are not empty.
    ///
    /// # Returns
    ///
    /// `Ok(())` if all required fields are present, `Err` with a descriptive message otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// let valid = OvhClient {
    ///     application_key: "app_key".into(),
    ///     application_secret: "app_secret".into(),
    ///     consumer_key: "consumer_key".into(),
    ///     server: "https://eu.api.ovh.com/1.0".into(),
    /// };
    /// assert!(valid.validate_config().is_ok());
    ///
    /// let invalid = OvhClient {
    ///     application_key: "".into(),
    ///     application_secret: "app_secret".into(),
    ///     consumer_key: "consumer_key".into(),
    ///     server: "https://eu.api.ovh.com/1.0".into(),
    /// };
    /// assert!(invalid.validate_config().is_err());
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.application_key.is_empty() {
            return Err("application key (username) is required for OVH".into());
        }
        if self.application_secret.is_empty() {
            return Err("application secret (password) is required for OVH".into());
        }
        if self.consumer_key.is_empty() {
            return Err("consumer key (api_token) is required for OVH".into());
        }
        Ok(())
    }

    /// Provider identifier for this client.
    ///
    /// # Returns
    ///
    /// `&str` with the provider name — `"OVH"`.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = OvhClient {
    ///     application_key: String::new(),
    ///     application_secret: String::new(),
    ///     consumer_key: String::new(),
    ///     server: String::from("https://eu.api.ovh.com/1.0"),
    /// };
    /// assert_eq!(client.provider_name(), "OVH");
    /// ```
    fn provider_name(&self) -> &str {
        "OVH"
    }
}