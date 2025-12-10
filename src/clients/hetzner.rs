use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

pub struct HetznerClient {
    api_token: String,
    zone_id: String,
    server: String,
}

impl HetznerClient {
    /// Creates a new HetznerClient from the provided configuration.
    ///
    /// The function extracts the API token and zone identifier from `config` and sets the DNS
    /// server URL to the configured value or `"https://dns.hetzner.com"` when none is provided.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration does not contain an API token or a zone identifier.
    /// The error messages are:
    /// - `"Hetzner requires API token (use password or api_token)"`
    /// - `"Hetzner requires zone_id (domain name)"`
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::clients::hetzner::HetznerClient;
    /// use crate::Config;
    ///
    /// let cfg = Config {
    ///     password: Some("token123".to_string()),
    ///     zone: Some("example.com".to_string()),
    ///     server: None,
    ///     ..Default::default()
    /// };
    ///
    /// let client = HetznerClient::new(&cfg).unwrap();
    /// assert_eq!(client.provider_name(), "Hetzner");
    /// ```
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let api_token = config.password.as_ref()
            .or(config.password.as_ref())
            .ok_or("Hetzner requires API token (use password or api_token)")?
            .clone();
        let zone_id = config.zone.as_ref()
            .ok_or("Hetzner requires zone_id (domain name)")?
            .clone();
        let server = config.server.clone()
            .unwrap_or_else(|| "https://dns.hetzner.com".to_string());

        Ok(Self {
            api_token,
            zone_id,
            server,
        })
    }

    /// Fetches the DNS record ID for `hostname` with the specified `record_type` in the client's zone.
    ///
    /// Returns the record's ID as a `String` if a matching record is found. Returns an error if the
    /// HTTP request fails, the response cannot be parsed, or no matching record exists.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::error::Error;
    /// # fn try_example() -> Result<(), Box<dyn Error>> {
    /// let client = HetznerClient {
    ///     api_token: "token".into(),
    ///     zone_id: "zone".into(),
    ///     server: "https://dns.hetzner.com".into(),
    /// };
    /// let record_id = client.get_record_id("www", "A")?;
    /// println!("found record id: {}", record_id);
    /// # Ok(())
    /// # }
    /// ```
    fn get_record_id(&self, hostname: &str, record_type: &str) -> Result<String, Box<dyn Error>> {
        let url = format!("{}/records?zone_id={}", self.server, self.zone_id);
        
        let response = minreq::get(&url)
            .with_header("Auth-API-Token", &self.api_token)
            .send()?;

        if response.status_code != 200 {
            return Err(format!("Failed to fetch records: HTTP {}", response.status_code).into());
        }

        let json: serde_json::Value = response.json()?;
        
        if let Some(records) = json["records"].as_array() {
            for record in records {
                if record["name"].as_str() == Some(hostname) 
                    && record["type"].as_str() == Some(record_type) {
                    if let Some(id) = record["id"].as_str() {
                        return Ok(id.to_string());
                    }
                }
            }
        }

        Err(format!("Record {} not found", hostname).into())
    }
}

impl DnsClient for HetznerClient {
    /// Updates an existing DNS record for the given hostname to the provided IP address.
    ///
    /// Chooses "A" for IPv4 and "AAAA" for IPv6, looks up the record ID for the hostname and type,
    /// and sends an authenticated PUT request to replace the record value and TTL.
    ///
    /// # Returns
    ///
    /// `Ok(())` on successful update; `Err` if the record cannot be found, the HTTP request fails,
    /// or the server returns a non-200 status.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::net::IpAddr;
    /// # // Assume `client` is a properly constructed HetznerClient in scope.
    /// # let client = /* HetznerClient::new(&config).unwrap() */ unreachable!();
    /// let ip: IpAddr = "203.0.113.5".parse().unwrap();
    /// // Update "www" to the given IPv4 address
    /// let _ = client.update_record("www", ip);
    /// ```
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        // Determine record type based on IP version
        let record_type = match ip {
            IpAddr::V4(_) => "A",
            IpAddr::V6(_) => "AAAA",
        };
        
        log::info!("Fetching {} record ID for {}", record_type, hostname);
        let record_id = self.get_record_id(hostname, record_type)?;
        
        let url = format!("{}/records/{}", self.server, record_id);
        
        let payload = serde_json::json!({
            "value": ip.to_string(),
            "ttl": 60,
            "type": record_type,
            "name": hostname,
            "zone_id": self.zone_id
        });

        log::info!("Updating {} to {}", hostname, ip);
        
        let response = minreq::put(&url)
            .with_header("Auth-API-Token", &self.api_token)
            .with_header("Content-Type", "application/json")
            .with_json(&payload)?
            .send()?;

        if response.status_code == 200 {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else {
            Err(format!("Failed to update record: HTTP {}", response.status_code).into())
        }
    }

    /// Validates that the client has the required Hetzner API token and zone identifier.
    ///
    /// Returns `Ok(())` if both the API token and zone_id are non-empty; returns an `Err` with a
    /// descriptive message if either value is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = HetznerClient { api_token: "token".into(), zone_id: "example.com".into(), server: "https://dns.hetzner.com".into() };
    /// assert!(client.validate_config().is_ok());
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.api_token.is_empty() {
            return Err("Hetzner API token cannot be empty".into());
        }
        if self.zone_id.is_empty() {
            return Err("Hetzner zone_id cannot be empty".into());
        }
        Ok(())
    }

    /// Name of the DNS provider represented by this client.
    ///
    /// # Returns
    ///
    /// The provider name, `"Hetzner"`.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = HetznerClient { api_token: String::new(), zone_id: String::new(), server: String::new() };
    /// assert_eq!(client.provider_name(), "Hetzner");
    /// ```
    fn provider_name(&self) -> &str {
        "Hetzner"
    }
}