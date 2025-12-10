use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;
use serde_json::json;

/// Gandi client - https://api.gandi.net/docs/livedns/
pub struct GandiClient {
    api_key: String,
    server: String,
}

impl GandiClient {
    /// Constructs a GandiClient from the provided configuration.
    ///
    /// The API key is read from `config.password` and an error is returned if it is missing.
    /// The server URL is taken from `config.server` or defaults to "https://api.gandi.net".
    ///
    /// # Examples
    ///
    /// ```
    /// let cfg = Config { password: Some("token".to_string()), server: None };
    /// let client = GandiClient::new(&cfg).unwrap();
    /// assert_eq!(client.provider_name(), "Gandi");
    /// ```
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let api_key = config.password.as_ref()
            .or(config.password.as_ref())
            .ok_or("api_token or password is required for Gandi")?
            .clone();
        
        let server = config.server.as_ref()
            .cloned()
            .unwrap_or_else(|| "https://api.gandi.net".to_string());

        Ok(GandiClient { api_key, server })
    }

    /// Parse a hostname into the DNS record name and the domain (last two labels).
    ///
    /// The returned tuple is `(name, domain)`. `name` is the record label to use in DNS updates;
    /// when the hostname is a two‑label domain (e.g., `example.com`) `name` is `"@"` to denote the apex.
    /// `domain` is formed from the last two labels of the hostname (e.g., `example.com`).
    ///
    /// # Examples
    ///
    /// ```
    /// // name is the left-most label(s); domain is the last two labels
    /// let client = GandiClient { api_key: String::new(), server: String::new() };
    /// assert_eq!(client.parse_hostname("www.example.com"), ("www".to_string(), "example.com".to_string()));
    /// assert_eq!(client.parse_hostname("example.com"), ("@".to_string(), "example.com".to_string()));
    /// // name may contain dots when there are more than three labels
    /// assert_eq!(client.parse_hostname("a.b.c.example.com"), ("a.b.c".to_string(), "example.com".to_string()));
    /// ```
    fn parse_hostname(&self, hostname: &str) -> (String, String) {
        let parts: Vec<&str> = hostname.rsplitn(3, '.').collect();
        if parts.len() >= 3 {
            let domain = format!("{}.{}", parts[1], parts[0]);
            let name = parts[2];
            (name.to_string(), domain)
        } else {
            ("@".to_string(), hostname.to_string())
        }
    }
}

impl DnsClient for GandiClient {
    /// Update the A record for a hostname in Gandi LiveDNS.
    ///
    /// Sends a PUT request to Gandi's LiveDNS API to set the A record for `hostname` to `ip`.
    ///
    /// # Parameters
    ///
    /// - `hostname`: Fully qualified hostname to update (e.g. "sub.example.com").
    /// - `ip`: IPv4 or IPv6 address to assign to the A record.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the record was successfully created or updated (HTTP 200 or 201), `Err` if the
    /// API returns an error status or if the request/serialization fails.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::net::IpAddr;
    /// # // Constructing a client is omitted; this example demonstrates usage only.
    /// # let client = /* GandiClient::new(&config).unwrap() */ panic!();
    /// let hostname = "host.example.com";
    /// let ip: IpAddr = "203.0.113.5".parse().unwrap();
    /// let _ = client.update_record(hostname, ip);
    /// ```
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let (name, domain) = self.parse_hostname(hostname);
        
        let url = format!(
            "{}/v5/livedns/domains/{}/records/{}/A",
            self.server, domain, name
        );

        let body = json!({
            "rrset_values": [ip.to_string()],
            "rrset_ttl": 300
        });

        log::info!("Updating {} with Gandi", hostname);

        let response = minreq::put(&url)
            .with_header("Authorization", format!("Apikey {}", self.api_key))
            .with_header("Content-Type", "application/json")
            .with_json(&body)?
            .send()?;

        let status_code = response.status_code;

        if status_code == 200 || status_code == 201 {
            log::info!("DNS record for {} successfully updated to {}", hostname, ip);
            Ok(())
        } else {
            let body = response.as_str().unwrap_or("unknown error");
            Err(format!("Gandi API error ({}): {}", status_code, body).into())
        }
    }

    /// Validates that the client is configured with a non-empty API key.
    ///
    /// # Errors
    ///
    /// Returns an `Err` if the client's `api_key` is an empty string.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = GandiClient { api_key: "key".to_string(), server: "https://api.gandi.net".to_string() };
    /// assert!(client.validate_config().is_ok());
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.api_key.is_empty() {
            return Err("API key is required for Gandi".into());
        }
        Ok(())
    }

    /// Provider name for this client.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = GandiClient { api_key: String::from("key"), server: String::from("https://api.gandi.net") };
    /// assert_eq!(client.provider_name(), "Gandi");
    /// ```
    fn provider_name(&self) -> &str {
        "Gandi"
    }
}