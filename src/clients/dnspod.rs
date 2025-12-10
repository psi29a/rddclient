use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

/// DNSPod DNS client
/// Uses DNSPod token-based API
pub struct DnspodClient {
    server: String,
    token: String,
}

impl DnspodClient {
    /// Create a new DnspodClient from the provided configuration.
    ///
    /// The returned client is configured with the API token from `config.password` and uses
    /// `config.server` if present; otherwise it defaults to "https://dnsapi.cn".
    ///
    /// # Examples
    ///
    /// ```
    /// let config = Config { server: None, password: Some("api_token_value".to_string()) };
    /// let client = DnspodClient::new(&config).unwrap();
    /// assert_eq!(client.provider_name(), "DNSPod");
    /// ```
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let token = config.password.as_ref()
            .ok_or("api_token is required for DNSPod")?
            .clone();
        
        let server = config.server.clone()
            .unwrap_or_else(|| "https://dnsapi.cn".to_string());

        Ok(DnspodClient {
            server,
            token,
        })
    }
}

impl DnsClient for DnspodClient {
    /// Update the DNS record for `hostname` to the given `ip` using DNSPod's DDNS endpoint.
    ///
    /// Attempts to determine the record type from `ip` ("A" for IPv4, "AAAA" for IPv6), derives the
    /// domain and subdomain from `hostname`, and sends a POST to the DNSPod `Record.Ddns` API with the
    /// client's token.
    ///
    /// # Errors
    ///
    /// Returns an `Err` when:
    /// - `hostname` cannot be parsed into a domain and subdomain (invalid format),
    /// - the HTTP request fails or returns a non-200 status,
    /// - DNSPod returns an error response,
    /// - or the response body is not recognized as a success.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::IpAddr;
    /// let client = crate::clients::dnspod::DnspodClient { server: "https://dnsapi.cn".into(), token: "token,value".into() };
    /// let ip: IpAddr = "1.2.3.4".parse().unwrap();
    /// client.update_record("sub.example.com", ip).unwrap();
    /// ```
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let record_type = match ip {
            IpAddr::V4(_) => "A",
            IpAddr::V6(_) => "AAAA",
        };

        // Split hostname into subdomain and domain
        let parts: Vec<&str> = hostname.rsplitn(3, '.').collect();
        if parts.len() < 2 {
            return Err("Invalid hostname format".into());
        }
        let domain = format!("{}.{}", parts[1], parts[0]);
        let subdomain = if parts.len() >= 3 {
            parts[2]
        } else {
            "@"
        };

        log::info!("Updating {} with DNSPod", hostname);

        // DNSPod API endpoint
        let url = format!("{}/Record.Ddns", self.server);

        let response = minreq::post(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .with_header("Content-Type", "application/x-www-form-urlencoded")
            .with_body(format!(
                "login_token={}&format=json&domain={}&sub_domain={}&record_type={}&value={}",
                self.token,
                domain,
                subdomain,
                record_type,
                ip
            ))
            .send()?;

        let status_code = response.status_code;
        let body = response.as_str()?.trim();

        log::debug!("Response status: {}, body: {}", status_code, body);

        if status_code != 200 {
            return Err(format!("HTTP error: {}", status_code).into());
        }

        // Parse JSON response
        if body.contains(r#""code":"1""#) || body.contains("success") {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else if body.contains(r#""message":"#) {
            // Extract error message
            Err(format!("DNSPod error: {}", body).into())
        } else {
            Err(format!("Unexpected response: {}", body).into())
        }
    }

    /// Validate that the DNSPod client has a configured API token.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the API token is set, `Err` with an error message otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = DnspodClient { server: "https://dnsapi.cn".into(), token: "token".into() };
    /// client.validate_config().unwrap();
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.token.is_empty() {
            return Err("api_token is required for DNSPod".into());
        }
        Ok(())
    }

    /// Returns the provider name for this DNS client.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = DnspodClient { server: String::new(), token: String::new() };
    /// assert_eq!(client.provider_name(), "DNSPod");
    /// ```
    fn provider_name(&self) -> &str {
        "DNSPod"
    }
}