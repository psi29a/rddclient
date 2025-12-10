use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;
use base64::{Engine as _, engine::general_purpose};

pub struct MythicbeastsClient {
    username: String,
    password: String,
    server: String,
}

impl MythicbeastsClient {
    /// Creates a new MythicbeastsClient from the given configuration.
    ///
    /// The `config` must provide non-empty `login` and `password` values; `server` is optional and
    /// defaults to `"api.mythic-beasts.com"` when not present.
    ///
    /// # Errors
    ///
    /// Returns an `Err` if `login` is missing with message `"username is required for Mythic Beasts"`,
    /// or if `password` is missing with message `"password is required for Mythic Beasts"`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// // Construct a Config with `login` and `password` set, then create the client.
    /// // The exact Config construction depends on the crate's Config type.
    /// let config = /* build or obtain a Config with login and password */ ;
    /// let client = MythicbeastsClient::new(&config)?;
    /// ```
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let username = config.login.as_ref()
            .ok_or("username is required for Mythic Beasts")?;
        let password = config.password.as_ref()
            .ok_or("password is required for Mythic Beasts")?;
        let server = config.server.as_deref()
            .unwrap_or("api.mythic-beasts.com");

        Ok(MythicbeastsClient {
            username: username.to_string(),
            password: password.to_string(),
            server: server.to_string(),
        })
    }
}

impl DnsClient for MythicbeastsClient {
    /// Update the DNS record for `hostname` at Mythic Beasts to the given `ip`.
    ///
    /// Sends an authenticated POST to the provider's IPv4 or IPv6 dynamic DNS endpoint and treats HTTP 200 as success.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the provider accepted the update (HTTP 200), `Err` containing a descriptive message otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::IpAddr;
    /// let client = MythicbeastsClient {
    ///     username: "user".into(),
    ///     password: "pass".into(),
    ///     server: "api.mythic-beasts.com".into(),
    /// };
    /// let ip: IpAddr = "203.0.113.45".parse().unwrap();
    /// let _ = client.update_record("host.example.com", ip);
    /// ```
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        log::info!("Updating Mythic Beasts record for {} to {}", hostname, ip);

        // Mythic Beasts uses separate IPv4 and IPv6 endpoints
        let (ipv, subdomain) = match ip {
            IpAddr::V4(_) => ("4", "ipv4"),
            IpAddr::V6(_) => ("6", "ipv6"),
        };

        let url = format!(
            "https://{}.{}/dns/v2/dynamic/{}",
            subdomain, self.server, hostname
        );

        let auth = general_purpose::STANDARD.encode(format!("{}:{}", self.username, self.password));
        let response = minreq::post(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .with_header("Authorization", format!("Basic {}", auth))
            .send()?;

        if response.status_code == 200 {
            log::info!("Successfully updated IPv{} record for {} to {}", ipv, hostname, ip);
            Ok(())
        } else {
            let body = response.as_str().unwrap_or("No response body");
            Err(format!(
                "Mythic Beasts API error: HTTP {} - {}",
                response.status_code, body
            )
            .into())
        }
    }

    /// Validates that the client's username and password are present.
    ///
    /// # Errors
    ///
    /// Returns an error if the username is an empty string or if the password is an empty string.
    ///
    /// # Examples
    ///
    /// ```
    /// let ok = MythicbeastsClient { username: "user".into(), password: "pass".into(), server: "api".into() };
    /// assert!(ok.validate_config().is_ok());
    ///
    /// let no_user = MythicbeastsClient { username: "".into(), password: "pass".into(), server: "api".into() };
    /// assert!(no_user.validate_config().is_err());
    ///
    /// let no_pass = MythicbeastsClient { username: "user".into(), password: "".into(), server: "api".into() };
    /// assert!(no_pass.validate_config().is_err());
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.username.is_empty() {
            return Err("Mythic Beasts username cannot be empty".into());
        }
        if self.password.is_empty() {
            return Err("Mythic Beasts password cannot be empty".into());
        }
        Ok(())
    }

    /// Get the DNS provider's display name.
    ///
    /// The returned value is the provider name as a static string.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = MythicbeastsClient { username: String::new(), password: String::new(), server: String::new() };
    /// assert_eq!(client.provider_name(), "Mythic Beasts");
    /// ```
    fn provider_name(&self) -> &'static str {
        "Mythic Beasts"
    }
}