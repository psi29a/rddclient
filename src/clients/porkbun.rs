use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;
use serde_json::json;

/// Porkbun client - https://porkbun.com/api/json/v3/documentation
pub struct PorkbunClient {
    api_key: String,
    secret_key: String,
    server: String,
}

impl PorkbunClient {
    /// Create a new `PorkbunClient` using credentials and optional server from `config`.
    ///
    /// Errors if `config.login` is missing (returns an error with message
    /// "username (API key) is required for Porkbun") or if `config.password` is missing
    /// (returns an error with message "password (secret key) is required for Porkbun").
    /// If `config.server` is not provided, the Porkbun API default "https://porkbun.com/api/json/v3"
    /// is used.
    ///
    /// # Examples
    ///
    /// ```
    /// // prepare a Config with `login` and `password` set
    /// let config = Config {
    ///     login: Some("api-key".to_string()),
    ///     password: Some("secret-key".to_string()),
    ///     server: None,
    ///     /* other fields if any */
    /// };
    ///
    /// let client = PorkbunClient::new(&config).unwrap();
    /// assert_eq!(client.provider_name(), "Porkbun");
    /// ```
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let api_key = config.login.as_ref()
            .ok_or("username (API key) is required for Porkbun")?
            .clone();
        let secret_key = config.password.as_ref()
            .ok_or("password (secret key) is required for Porkbun")?
            .clone();
        
        let server = config.server.as_ref()
            .cloned()
            .unwrap_or_else(|| "https://porkbun.com/api/json/v3".to_string());

        Ok(PorkbunClient {
            api_key,
            secret_key,
            server,
        })
    }

    /// Extracts the subdomain label and registrable domain from a hostname.
    ///
    /// If the hostname contains at least three dot-separated components (from the right),
    /// the returned tuple is (subdomain_label, registrable_domain) where:
    /// - `subdomain_label` is the immediate label left of the registrable domain,
    /// - `registrable_domain` is the last two labels joined by a dot (e.g., "example.com").
    ///
    /// If the hostname contains fewer than three components, the function returns
    /// an empty `String` for the subdomain label and the original `hostname` as the domain.
    ///
    /// # Examples
    ///
    /// ```
    /// // hostname with subdomain
    /// let (name, domain) = PorkbunClient::parse_hostname(&PorkbunClient {
    ///     api_key: String::new(),
    ///     secret_key: String::new(),
    ///     server: String::new(),
    /// }, "www.example.com");
    /// assert_eq!(name, "www");
    /// assert_eq!(domain, "example.com");
    ///
    /// // hostname without subdomain
    /// let (name, domain) = PorkbunClient::parse_hostname(&PorkbunClient {
    ///     api_key: String::new(),
    ///     secret_key: String::new(),
    ///     server: String::new(),
    /// }, "example.com");
    /// assert_eq!(name, "");
    /// assert_eq!(domain, "example.com");
    /// ```
    fn parse_hostname(&self, hostname: &str) -> (String, String) {
        let parts: Vec<&str> = hostname.rsplitn(3, '.').collect();
        if parts.len() >= 3 {
            let domain = format!("{}.{}", parts[1], parts[0]);
            let name = parts[2];
            (name.to_string(), domain)
        } else {
            ("".to_string(), hostname.to_string())
        }
    }
}

impl DnsClient for PorkbunClient {
    /// Updates the DNS record for the given hostname to the provided IP address using the Porkbun API.
    ///
    /// Chooses the DNS record type (`A` or `AAAA`) based on the IP version, includes the hostname's
    /// subdomain when present, and returns `Ok(())` when the Porkbun API reports success.
    /// On failure returns an `Err` containing the API error message.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::IpAddr;
    /// // Assume `client` is a previously constructed PorkbunClient
    /// // let client = PorkbunClient::new(&config).unwrap();
    /// let ip: IpAddr = "203.0.113.42".parse().unwrap();
    /// let _ = client.update_record("sub.example.com", ip);
    /// ```
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let (name, domain) = self.parse_hostname(hostname);
        
        // Determine record type based on IP version
        let record_type = match ip {
            IpAddr::V4(_) => "A",
            IpAddr::V6(_) => "AAAA",
        };
        
        let url = format!("{}/dns/editByNameType/{}/{}", self.server, domain, record_type);
        
        let subdomain = if name.is_empty() { None } else { Some(name.as_str()) };

        let mut body = json!({
            "apikey": self.api_key,
            "secretapikey": self.secret_key,
            "content": ip.to_string(),
            "ttl": "600"
        });

        if let Some(sub) = subdomain {
            body["name"] = json!(sub);
        }

        log::info!("Updating {} with Porkbun", hostname);

        let response = minreq::post(&url)
            .with_header("Content-Type", "application/json")
            .with_json(&body)?
            .send()?;

        let json: serde_json::Value = response.json()?;

        if json["status"] == "SUCCESS" {
            log::info!("DNS record for {} successfully updated to {}", hostname, ip);
            Ok(())
        } else {
            let message = json["message"].as_str().unwrap_or("unknown error");
            Err(format!("Porkbun API error: {}", message).into())
        }
    }

    /// Validates that the client has both API and secret keys configured.
    ///
    /// Returns `Ok(())` if both `api_key` and `secret_key` are non-empty, `Err` with a descriptive message otherwise.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let client = PorkbunClient::new(&config).unwrap();
    /// client.validate_config()?;
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.api_key.is_empty() {
            return Err("API key (username) is required for Porkbun".into());
        }
        if self.secret_key.is_empty() {
            return Err("Secret key (password) is required for Porkbun".into());
        }
        Ok(())
    }

    /// Get the DNS provider identifier for this client.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// // given a `PorkbunClient` instance `client`
    /// let name = client.provider_name();
    /// assert_eq!(name, "Porkbun");
    /// ```
    fn provider_name(&self) -> &str {
        "Porkbun"
    }
}