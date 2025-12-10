use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

/// Hurricane Electric (HE.net) client - https://dns.he.net/
pub struct HurricaneElectricClient {
    password: String,
    server: String,
}

impl HurricaneElectricClient {
    /// Creates a new HurricaneElectricClient from the given configuration.
    ///
    /// The function requires the config to contain a `password`; if `server` is absent,
    /// it defaults to "https://dyn.dns.he.net/nic/update".
    ///
    /// # Examples
    ///
    /// ```
    /// let cfg = Config {
    ///     password: Some("secret".to_string()),
    ///     server: None,
    /// };
    /// let client = HurricaneElectricClient::new(&cfg).unwrap();
    /// assert_eq!(client.provider_name(), "Hurricane Electric");
    /// ```
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let password = config.password.as_ref()
            .ok_or("password is required for Hurricane Electric")?
            .clone();
        
        let server = config.server.as_ref()
            .cloned()
            .unwrap_or_else(|| "https://dyn.dns.he.net/nic/update".to_string());

        Ok(HurricaneElectricClient { password, server })
    }
}

impl DnsClient for HurricaneElectricClient {
    /// Update the DNS A record for a hostname at Hurricane Electric.
    ///
    /// Attempts to set the hostname's address to `ip` using the configured server and password.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the update response indicates success; `Err` with a descriptive message if authentication fails, the hostname is not a fully-qualified domain name, or the provider returns an unexpected response.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::IpAddr;
    /// // Obtain a configured `HurricaneElectricClient` (example omitted).
    /// let client = /* HurricaneElectricClient instance */ unimplemented!();
    /// client.update_record("host.example.com", "1.2.3.4".parse::<IpAddr>().unwrap()).unwrap();
    /// ```
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let url = format!(
            "{}?hostname={}&password={}&myip={}",
            self.server, hostname, self.password, ip
        );

        log::info!("Updating {} with Hurricane Electric", hostname);

        let response = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .send()?;

        let body = response.as_str()?.trim();

        // HE.net returns various responses
        if body.contains("good") || body.contains("nochg") {
            log::info!("DNS record for {} successfully updated to {}", hostname, ip);
            Ok(())
        } else if body.contains("badauth") {
            Err("Bad authentication - check your password".into())
        } else if body.contains("notfqdn") {
            Err("Not a fully-qualified domain name".into())
        } else {
            Err(format!("Unexpected response: {}", body).into())
        }
    }

    /// Ensures the client has a non-empty password.
    ///
    /// Returns an error if the password is empty, otherwise returns `Ok(())`.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = HurricaneElectricClient {
    ///     password: "secret".into(),
    ///     server: "https://dyn.dns.he.net/nic/update".into(),
    /// };
    /// assert!(client.validate_config().is_ok());
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.password.is_empty() {
            return Err("password is required for Hurricane Electric".into());
        }
        Ok(())
    }

    /// Identifies the DNS provider implemented by this client.
    ///
    /// # Returns
    ///
    /// The provider name: `"Hurricane Electric"`.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = HurricaneElectricClient { password: String::from("pw"), server: String::from("https://dyn.dns.he.net/nic/update") };
    /// assert_eq!(client.provider_name(), "Hurricane Electric");
    /// ```
    fn provider_name(&self) -> &str {
        "Hurricane Electric"
    }
}