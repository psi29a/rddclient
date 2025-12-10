use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

/// Namecheap Dynamic DNS client
pub struct NamecheapClient {
    server: String,
    domain: String,
    password: String,
}

impl NamecheapClient {
    /// Constructs a new `NamecheapClient` from the provided `Config`.
    ///
    /// The `config` must provide `login` (used as the domain) and `password`. If `server` is
    /// not specified, the Namecheap dynamic DNS endpoint `dynamicdns.park-your-domain.com` is used.
    /// Returns an error if `login` or `password` is missing.
    ///
    /// # Examples
    ///
    /// ```
    /// let cfg = Config {
    ///     login: Some("example.com".to_string()),
    ///     password: Some("secret".to_string()),
    ///     server: None,
    ///     ..Default::default()
    /// };
    /// let client = NamecheapClient::new(&cfg).unwrap();
    /// assert_eq!(client.provider_name(), "Namecheap");
    /// ```
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        // For Namecheap, username is the domain name
        let domain = config.login.as_ref()
            .ok_or("username (domain) is required for Namecheap")?
            .clone();
        let password = config.password.as_ref()
            .ok_or("password is required for Namecheap")?
            .clone();
        
        let server = config.server.as_ref()
            .cloned()
            .unwrap_or_else(|| "dynamicdns.park-your-domain.com".to_string());

        Ok(NamecheapClient {
            server,
            domain,
            password,
        })
    }
}

impl DnsClient for NamecheapClient {
    /// Updates the DNS A record for `hostname` to the provided `ip` using Namecheap's Dynamic DNS API.
    ///
    /// The function determines the sub-host sent to Namecheap as:
    /// - the label before `.domain` when `hostname` ends with `.<domain>`,
    /// - `@` when `hostname` equals the configured domain,
    /// - otherwise the full `hostname`.
    ///
    /// On success returns `Ok(())`. On failure returns `Err` containing either an HTTP error description or the Namecheap error message extracted from the provider response.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::IpAddr;
    /// use std::str::FromStr;
    ///
    /// let client = NamecheapClient {
    ///     server: "dynamicdns.park-your-domain.com".into(),
    ///     domain: "example.com".into(),
    ///     password: "secret".into(),
    /// };
    ///
    /// // Performs a dynamic DNS update (network request) — not executed in doctest.
    /// let _ = client.update_record("www.example.com", IpAddr::from_str("1.2.3.4").unwrap());
    /// ```
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        // Extract subdomain from hostname
        // e.g., "www.example.com" with domain "example.com" -> "www"
        let host = if hostname.ends_with(&format!(".{}", self.domain)) {
            hostname.trim_end_matches(&format!(".{}", self.domain))
        } else if hostname == self.domain {
            "@"
        } else {
            hostname
        };

        let url = format!(
            "https://{}/update?host={}&domain={}&password={}&ip={}",
            self.server, host, self.domain, self.password, ip
        );

        log::info!("Updating {} with Namecheap", hostname);

        let response = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .send()?;

        let status_code = response.status_code;
        let body = response.as_str()?;

        log::debug!("Response status: {}, body: {}", status_code, body);

        if status_code != 200 {
            return Err(format!("HTTP error: {}", status_code).into());
        }

        // Namecheap returns XML response
        // Success: <ErrCount>0</ErrCount>
        // Failure: <ErrCount>1</ErrCount> (or higher)
        if body.contains("<ErrCount>0") {
            log::info!("DNS record for {} successfully updated to {}", hostname, ip);
            Ok(())
        } else {
            // Try to extract error message
            if let Some(start) = body.find("<Err1>") {
                if let Some(end) = body[start..].find("</Err1>") {
                    let error_msg = &body[start + 6..start + end];
                    return Err(format!("Namecheap error: {}", error_msg).into());
                }
            }
            Err(format!("Update failed: {}", body).into())
        }
    }

    /// Validates that the client has both a domain (username) and a password configured.
    ///
    /// Returns `Ok(())` if both `domain` and `password` are non-empty; returns `Err` with a
    /// descriptive message otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// let good = NamecheapClient {
    ///     server: "dynamicdns.park-your-domain.com".to_string(),
    ///     domain: "example.com".to_string(),
    ///     password: "secret".to_string(),
    /// };
    /// assert!(good.validate_config().is_ok());
    ///
    /// let bad = NamecheapClient {
    ///     server: "dynamicdns.park-your-domain.com".to_string(),
    ///     domain: "".to_string(),
    ///     password: "".to_string(),
    /// };
    /// assert!(bad.validate_config().is_err());
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.domain.is_empty() {
            return Err("username (domain) is required for Namecheap".into());
        }
        if self.password.is_empty() {
            return Err("password is required for Namecheap".into());
        }
        Ok(())
    }

    /// Human-readable provider name for this DNS client.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = NamecheapClient { server: String::new(), domain: String::new(), password: String::new() };
    /// assert_eq!(client.provider_name(), "Namecheap");
    /// ```
    ///
    /// # Returns
    ///
    /// `&str` with the static provider name "Namecheap".
    fn provider_name(&self) -> &str {
        "Namecheap"
    }
}