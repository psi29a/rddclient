use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;
use serde_json::json;

/// GoDaddy client - https://developer.godaddy.com/doc/endpoint/domains
pub struct GoDaddyClient {
    api_key: String,
    api_secret: String,
    server: String,
}

impl GoDaddyClient {
    /// Creates a GoDaddyClient from the provided Config by extracting required credentials and an optional server URL.
    ///
    /// The function requires `config.login` (API key) and `config.password` (API secret). If `config.server` is omitted, the default
    /// "https://api.godaddy.com" is used.
    ///
    /// # Errors
    ///
    /// Returns an error if `login` or `password` are missing from the config.
    ///
    /// # Examples
    ///
    /// ```
    /// let cfg = Config {
    ///     login: Some("api_key".to_string()),
    ///     password: Some("api_secret".to_string()),
    ///     server: None,
    /// };
    /// let client = GoDaddyClient::new(&cfg).unwrap();
    /// assert_eq!(client.provider_name(), "GoDaddy");
    /// ```
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let api_key = config.login.as_ref()
            .ok_or("username (API key) is required for GoDaddy")?
            .clone();
        let api_secret = config.password.as_ref()
            .ok_or("password (API secret) is required for GoDaddy")?
            .clone();
        
        let server = config.server.as_ref()
            .cloned()
            .unwrap_or_else(|| "https://api.godaddy.com".to_string());

        Ok(GoDaddyClient {
            api_key,
            api_secret,
            server,
        })
    }

    /// Split a hostname into the DNS record name and its domain.
    ///
    /// If the hostname contains at least three dot-separated labels (e.g. "www.example.com"),
    /// the domain is the last two labels joined with a dot and the name is the remaining left-hand portion
    /// (e.g. returns ("www", "example.com")). If the hostname has fewer than three labels
    /// (e.g. "example.com" or "localhost"), the function treats the hostname as the root domain
    /// and returns ("@", hostname).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// // Given a GoDaddyClient instance `client`, call:
    /// let (name, domain) = client.parse_hostname("www.example.com");
    /// assert_eq!(name, "www");
    /// assert_eq!(domain, "example.com");
    ///
    /// let (name_root, domain_root) = client.parse_hostname("example.com");
    /// assert_eq!(name_root, "@");
    /// assert_eq!(domain_root, "example.com");
    /// ```
    fn parse_hostname(&self, hostname: &str) -> (String, String) {
        // Split hostname into domain and record name
        // e.g., "www.example.com" -> ("www", "example.com")
        let parts: Vec<&str> = hostname.rsplitn(3, '.').collect();
        if parts.len() >= 3 {
            let domain = format!("{}.{}", parts[1], parts[0]);
            let name = parts[2];
            (name.to_string(), domain)
        } else {
            // Assume @ for root domain
            ("@".to_string(), hostname.to_string())
        }
    }
}

impl DnsClient for GoDaddyClient {
    /// Updates the DNS record for a hostname at GoDaddy to the provided IP address.
    ///
    /// Sends a PUT request to the GoDaddy Domains API to set an `A` record for IPv4
    /// or an `AAAA` record for IPv6 with a TTL of 600 seconds. Returns `Ok(())` when
    /// the API responds with HTTP 200; returns `Err` containing the status code and
    /// response body otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::IpAddr;
    ///
    /// // `client` is a `GoDaddyClient` previously constructed and configured.
    /// // let client = ...;
    /// // let res = client.update_record("sub.example.com", "1.2.3.4".parse::<IpAddr>().unwrap());
    /// // assert!(res.is_ok());
    /// ```
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let (name, domain) = self.parse_hostname(hostname);
        
        // Determine record type based on IP version
        let record_type = match ip {
            IpAddr::V4(_) => "A",
            IpAddr::V6(_) => "AAAA",
        };
        
        let url = format!(
            "{}/v1/domains/{}/records/{}/{}",
            self.server, domain, record_type, name
        );

        let body = json!([{
            "data": ip.to_string(),
            "ttl": 600
        }]);

        log::info!("Updating {} with GoDaddy", hostname);

        let response = minreq::put(&url)
            .with_header("Authorization", format!("sso-key {}:{}", self.api_key, self.api_secret))
            .with_header("Content-Type", "application/json")
            .with_json(&body)?
            .send()?;

        let status_code = response.status_code;

        if status_code == 200 {
            log::info!("DNS record for {} successfully updated to {}", hostname, ip);
            Ok(())
        } else {
            let body = response.as_str().unwrap_or("unknown error");
            Err(format!("GoDaddy API error ({}): {}", status_code, body).into())
        }
    }

    /// Ensures the client has both an API key and API secret configured.
    ///
    /// Returns `Ok(())` if both credentials are present, `Err` with a descriptive message otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = GoDaddyClient {
    ///     api_key: "key".to_string(),
    ///     api_secret: "secret".to_string(),
    ///     server: "https://api.godaddy.com".to_string(),
    /// };
    /// assert!(client.validate_config().is_ok());
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.api_key.is_empty() {
            return Err("API key (username) is required for GoDaddy".into());
        }
        if self.api_secret.is_empty() {
            return Err("API secret (password) is required for GoDaddy".into());
        }
        Ok(())
    }

    /// Returns the provider's display name.
    ///
    /// # Returns
    ///
    /// The string `"GoDaddy"`.
    ///
    /// # Examples
    ///
    /// ```
    /// // Given a GoDaddyClient instance `client`:
    /// // let name = client.provider_name();
    /// // assert_eq!(name, "GoDaddy");
    /// ```
    fn provider_name(&self) -> &str {
        "GoDaddy"
    }
}