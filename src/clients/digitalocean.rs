use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;
use serde_json::json;

/// DigitalOcean client - https://docs.digitalocean.com/reference/api/api-reference/#tag/Domain-Records
pub struct DigitalOceanClient {
    token: String,
    server: String,
}

impl DigitalOceanClient {
    /// Creates a DigitalOceanClient from the given configuration.
    ///
    /// The API token is read from `config.password`. The API server base URL is taken from
    /// `config.server` or defaults to "https://api.digitalocean.com" when not provided.
    ///
    /// # Errors
    ///
    /// Returns an `Err` if the configuration does not contain an API token.
    ///
    /// # Examples
    ///
    /// ```
    /// // Construct a Config with a password/token; fields depend on your Config definition.
    /// let cfg = Config {
    ///     password: Some("example-token".to_string()),
    ///     server: None,
    ///     ..Default::default()
    /// };
    /// let client = DigitalOceanClient::new(&cfg).unwrap();
    /// assert_eq!(client.provider_name(), "DigitalOcean");
    /// ```
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let token = config.password.as_ref()
            .or(config.password.as_ref())
            .ok_or("api_token or password is required for DigitalOcean")?
            .clone();
        
        let server = config.server.as_ref()
            .cloned()
            .unwrap_or_else(|| "https://api.digitalocean.com".to_string());

        Ok(DigitalOceanClient { token, server })
    }

    /// Split a full hostname into the DNS record name and the domain.
    ///
    /// If the hostname contains at least two dot-separated components (e.g. `www.example.com`),
    /// the record name is the left-most component immediately before the domain (e.g. `www`)
    /// and the domain is the last two components joined by a dot (e.g. `example.com`).
    /// If the hostname does not contain at least two dots, returns `"@"` as the record name
    /// and the original hostname as the domain.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// // Given a DigitalOceanClient instance `client`:
    /// // www.example.com -> ("www", "example.com")
    /// let result = client.parse_hostname("www.example.com");
    /// assert_eq!(result, ("www".to_string(), "example.com".to_string()));
    ///
    /// // example.com -> ("@", "example.com")
    /// let result = client.parse_hostname("example.com");
    /// assert_eq!(result, ("@".to_string(), "example.com".to_string()));
    /// ```
    fn parse_hostname(&self, hostname: &str) -> (String, String) {
        // Split hostname into record name and domain
        let parts: Vec<&str> = hostname.rsplitn(3, '.').collect();
        if parts.len() >= 3 {
            let domain = format!("{}.{}", parts[1], parts[0]);
            let name = parts[2];
            (name.to_string(), domain)
        } else {
            ("@".to_string(), hostname.to_string())
        }
    }

    /// Fetches the DigitalOcean DNS record ID for a given domain, record name, and record type.
    ///
    /// Looks up domain records via the DigitalOcean API and returns the matching record's `id`.
    ///
    /// # Returns
    ///
    /// The DNS record's ID as a `u64` if a matching record is found; otherwise returns an error describing that no matching record was found.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let client = DigitalOceanClient { token: "token".to_string(), server: "https://api.digitalocean.com".to_string() };
    /// let id = client.get_record_id("example.com", "www", "A").unwrap();
    /// println!("record id: {}", id);
    /// ```
    fn get_record_id(&self, domain: &str, name: &str, record_type: &str) -> Result<u64, Box<dyn Error>> {
        let url = format!("{}/v2/domains/{}/records", self.server, domain);

        let response = minreq::get(&url)
            .with_header("Authorization", format!("Bearer {}", self.token))
            .with_header("Content-Type", "application/json")
            .send()?;

        let json: serde_json::Value = response.json()?;
        
        if let Some(records) = json["domain_records"].as_array() {
            for record in records {
                if record["type"] == record_type && record["name"] == name {
                    if let Some(id) = record["id"].as_u64() {
                        return Ok(id);
                    }
                }
            }
        }

        Err(format!("No {} record found for {}.{}", record_type, name, domain).into())
    }
}

impl DnsClient for DigitalOceanClient {
    /// Updates the DNS record for `hostname` to the provided `ip` via the DigitalOcean API.
    ///
    /// This finds the domain and record ID for `hostname`, sets the record type to `A` for IPv4 or `AAAA` for IPv6,
    /// and updates the record's `data` field to the string form of `ip`. Returns an `Err` if the DNS record cannot
    /// be located or if the DigitalOcean API returns a non-200 response.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::net::IpAddr;
    /// # use my_crate::clients::digitalocean::DigitalOceanClient;
    /// # use my_crate::config::Config;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let cfg = Config { /* fill with valid token/server */ ..Default::default() };
    /// let client = DigitalOceanClient::new(&cfg)?;
    /// client.update_record("host.example.com", "203.0.113.42".parse::<IpAddr>()?)?;
    /// # Ok(()) }
    /// ```
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let (name, domain) = self.parse_hostname(hostname);
        
        // Determine record type based on IP version
        let record_type = match ip {
            IpAddr::V4(_) => "A",
            IpAddr::V6(_) => "AAAA",
        };
        
        log::info!("Updating {} with DigitalOcean ({})", hostname, record_type);
        
        let record_id = self.get_record_id(&domain, &name, record_type)?;
        
        let url = format!("{}/v2/domains/{}/records/{}", self.server, domain, record_id);

        let body = json!({
            "data": ip.to_string()
        });

        let response = minreq::put(&url)
            .with_header("Authorization", format!("Bearer {}", self.token))
            .with_header("Content-Type", "application/json")
            .with_json(&body)?
            .send()?;

        let status_code = response.status_code;

        if status_code == 200 {
            log::info!("DNS record for {} successfully updated to {}", hostname, ip);
            Ok(())
        } else {
            let body = response.as_str().unwrap_or("unknown error");
            Err(format!("DigitalOcean API error ({}): {}", status_code, body).into())
        }
    }

    /// Validates that the client is configured with a non-empty API token.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the token is present, `Err` containing an error message if the token is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = DigitalOceanClient { token: "token".into(), server: "https://api.digitalocean.com".into() };
    /// assert!(client.validate_config().is_ok());
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.token.is_empty() {
            return Err("API token is required for DigitalOcean".into());
        }
        Ok(())
    }

    /// DNS provider name for this client.
    ///
    /// # Returns
    ///
    /// The provider name `"DigitalOcean"`.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = DigitalOceanClient { token: String::new(), server: String::new() };
    /// assert_eq!(client.provider_name(), "DigitalOcean");
    /// ```
    fn provider_name(&self) -> &str {
        "DigitalOcean"
    }
}