use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

pub struct CloudnsClient {
    dynurl: String,
}

impl CloudnsClient {
    /// Create a new CloudnsClient using the dynamic URL from the provided config.
    ///
    /// The constructor uses `config.password` if present, otherwise `config.server`. Returns an error
    /// if neither contains a value.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::{clients::cloudns::CloudnsClient, Config};
    ///
    /// let cfg = Config { password: Some("https://dyn.example/update".into()), server: None, ..Default::default() };
    /// let client = CloudnsClient::new(&cfg).expect("should create client");
    /// ```
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        // ClouDNS uses a unique dynamic URL per host
        let dynurl = config.password.as_ref()
            .or(config.server.as_ref())
            .ok_or("ClouDNS requires dynurl (use password or server config)")?
            .clone();

        Ok(Self {
            dynurl,
        })
    }
}

impl DnsClient for CloudnsClient {
    /// Update the DNS record at the configured ClouDNS dynamic URL to the provided IP address.
    ///
    /// Sends an HTTP GET to the client's dynurl with a `myip` query parameter and interprets
    /// common ClouDNS response keywords to determine success.
    ///
    /// # Returns
    ///
    /// `Ok(())` on a detected successful update; `Err` containing a descriptive message on HTTP
    /// errors or explicit failure responses from the provider.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::IpAddr;
    /// let client = CloudnsClient { dynurl: "https://example.com/update?auth=token".into() };
    /// let ip: IpAddr = "203.0.113.42".parse().unwrap();
    /// assert!(client.update_record("ignored-hostname", ip).is_ok());
    /// ```
    fn update_record(&self, _hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        // ClouDNS dynurl already contains the hostname, just append IP
        let url = if self.dynurl.contains('?') {
            format!("{}&myip={}", self.dynurl, ip)
        } else {
            format!("{}?myip={}", self.dynurl, ip)
        };
        
        log::info!("Updating ClouDNS record to {}", ip);
        
        let response = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .send()?;

        if response.status_code != 200 {
            return Err(format!("HTTP error: {}", response.status_code).into());
        }

        let body = response.as_str()?;
        
        // ClouDNS typically returns success indicators
        if body.to_lowercase().contains("success") 
            || body.contains("good") 
            || body.contains("updated") {
            log::info!("Successfully updated to {}", ip);
            Ok(())
        } else if body.to_lowercase().contains("error") 
            || body.to_lowercase().contains("fail") {
            Err(format!("Update failed: {}", body).into())
        } else {
            // Assume success if no error indicator
            log::warn!("Unclear response, assuming success: {}", body);
            Ok(())
        }
    }

    /// Validates the configured ClouDNS dynamic URL.
    ///
    /// Ensures the client's `dynurl` is not empty and begins with `http://` or `https://`.
    ///
    /// # Returns
    ///
    /// `Ok(())` if `dynurl` passes validation; `Err` with a descriptive message otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = CloudnsClient { dynurl: "https://dyn.example.com/update".into() };
    /// assert!(client.validate_config().is_ok());
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.dynurl.is_empty() {
            return Err("ClouDNS dynurl cannot be empty".into());
        }
        if !self.dynurl.starts_with("http://") && !self.dynurl.starts_with("https://") {
            return Err("ClouDNS dynurl must start with http:// or https://".into());
        }
        Ok(())
    }

    /// Provider display name for this client.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = CloudnsClient { dynurl: "http://example".to_string() };
    /// assert_eq!(client.provider_name(), "ClouDNS");
    /// ```
    fn provider_name(&self) -> &str {
        "ClouDNS"
    }
}