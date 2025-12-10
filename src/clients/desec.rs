use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

/// deSEC DNS client
/// Uses deSEC REST API
pub struct DesecClient {
    server: String,
    token: String,
    domain: String,
}

impl DesecClient {
    /// Create a new deSEC DNS client from a Config.
    ///
    /// Validates that the config provides an API token and a domain (zone), and uses
    /// "https://update.dedyn.io" as the server URL when none is supplied in config.
    ///
    /// # Examples
    ///
    /// ```
    /// // Construct a Config with the expected fields.
    /// let cfg = Config {
    ///     password: Some("example-token".to_string()),
    ///     zone: Some("example.com".to_string()),
    ///     server: None,
    /// };
    /// let client = DesecClient::new(&cfg).expect("failed to create deSEC client");
    /// assert_eq!(client.provider_name(), "deSEC");
    /// ```
    ///
    /// # Returns
    ///
    /// A configured `DesecClient` on success; returns an error if `password` (api_token)
    /// or `zone` (domain) is missing from the provided `Config`.
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let token = config.password.as_ref()
            .ok_or("api_token is required for deSEC")?
            .clone();
        
        let domain = config.zone.as_ref()
            .ok_or("zone_id (domain) is required for deSEC")?
            .clone();
        
        let server = config.server.clone()
            .unwrap_or_else(|| "https://update.dedyn.io".to_string());

        Ok(DesecClient {
            server,
            token,
            domain,
        })
    }
}

impl DnsClient for DesecClient {
    /// Update the DNS record for a hostname at deSEC using the DynDNS2-compatible update endpoint.
    ///
    /// Performs a GET request to the configured deSEC server's /update endpoint using HTTP Basic
    /// authentication (domain:token). Interprets a 200 response whose body starts with `good` or
    /// `nochg` as success; known error responses such as `badauth` and `notfqdn` are mapped to
    /// descriptive errors.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the update succeeded (response body starts with `good` or `nochg`), `Err` with a
    /// descriptive message otherwise.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::IpAddr;
    ///
    /// let client = DesecClient {
    ///     server: "https://update.dedyn.io".into(),
    ///     token: "token".into(),
    ///     domain: "example.com".into(),
    /// };
    ///
    /// let ip: IpAddr = "1.2.3.4".parse().unwrap();
    /// client.update_record("host.example.com", ip).unwrap();
    /// ```
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        log::info!("Updating {} with deSEC", hostname);

        // Extract subdomain from hostname
        let subdomain = if hostname.ends_with(&self.domain) {
            hostname.strip_suffix(&format!(".{}", self.domain))
                .unwrap_or("")
        } else {
            hostname
        };

        // deSEC update endpoint (DynDNS2 compatible)
        let url = format!("{}/update", self.server);

        let auth = format!("{}:{}", self.domain, self.token);
        let encoded_auth = format!("Basic {}", base64::encode(&auth));

        let response = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .with_header("Authorization", &encoded_auth)
            .with_param("hostname", if subdomain.is_empty() { &self.domain } else { hostname })
            .with_param("myip", ip.to_string())
            .send()?;

        let status_code = response.status_code;
        let body = response.as_str()?.trim();

        log::debug!("Response status: {}, body: {}", status_code, body);

        if status_code != 200 {
            return Err(format!("HTTP error: {}", status_code).into());
        }

        // deSEC returns status codes similar to DynDNS2
        if body.starts_with("good") || body.starts_with("nochg") {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else if body.starts_with("badauth") {
            Err("Authentication failed".into())
        } else if body.starts_with("notfqdn") {
            Err("Invalid hostname".into())
        } else {
            Err(format!("Update failed: {}", body).into())
        }
    }

    /// Validates that the client has a non-empty API token and domain configured.
    ///
    /// Returns `Ok(())` when both token and domain are present.
    /// Returns an `Err` with the message `"api_token is required for deSEC"` if the token is empty,
    /// or `"zone_id (domain) is required for deSEC"` if the domain is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = DesecClient { server: "https://example".into(), token: "token".into(), domain: "example.com".into() };
    /// assert!(client.validate_config().is_ok());
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.token.is_empty() {
            return Err("api_token is required for deSEC".into());
        }
        if self.domain.is_empty() {
            return Err("zone_id (domain) is required for deSEC".into());
        }
        Ok(())
    }

    /// Provider identifier for this DNS client implementation.
    ///
    /// Returns the static provider name used to identify this client.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = DesecClient { server: String::new(), token: String::new(), domain: String::new() };
    /// assert_eq!(client.provider_name(), "deSEC");
    /// ```
    fn provider_name(&self) -> &str {
        "deSEC"
    }
}

mod base64 {
    use base64::{Engine as _, engine::general_purpose};
    
    /// Encodes a UTF-8 string to Base64 using the standard alphabet.
    ///
    /// The returned string is the Base64 representation of `data` using the standard
    /// character set (RFC 4648).
    ///
    /// # Examples
    ///
    /// ```
    /// let out = base64::encode("foo");
    /// assert_eq!(out, "Zm9v");
    /// ```
    pub fn encode(data: &str) -> String {
        general_purpose::STANDARD.encode(data.as_bytes())
    }
}