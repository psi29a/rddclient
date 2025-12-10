use crate::clients::DnsClient;
use crate::config::Config;
use base64::{Engine as _, engine::general_purpose};
use std::error::Error;
use std::net::IpAddr;

pub struct InwxClient {
    username: String,
    password: String,
    server: String,
}

impl InwxClient {
    /// Creates an `InwxClient` from the provided configuration by extracting credentials and the server URL.
    ///
    /// The function reads `login` and `password` from `config`; if either is missing it returns an error with the exact message
    /// `"INWX requires username"` or `"INWX requires password"`. If `server` is not set in the config, the default
    /// `"https://dyndns.inwx.com"` is used.
    ///
    /// # Examples
    ///
    /// ```
    /// let cfg = Config {
    ///     login: Some("user".to_string()),
    ///     password: Some("secret".to_string()),
    ///     server: None,
    ///     // ... other fields ...
    /// };
    /// let client = InwxClient::new(&cfg).unwrap();
    /// assert_eq!(client.provider_name(), "INWX");
    /// ```
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let username = config.login.as_ref()
            .ok_or("INWX requires username")?
            .clone();
        let password = config.password.as_ref()
            .ok_or("INWX requires password")?
            .clone();
        let server = config.server.clone()
            .unwrap_or_else(|| "https://dyndns.inwx.com".to_string());

        Ok(Self {
            username,
            password,
            server,
        })
    }
}

impl DnsClient for InwxClient {
    /// Update the DNS A record for `hostname` using the INWX DynDNS update API.
    ///
    /// This performs an authenticated DynDNS2-style update against the client's configured
    /// server and interprets the provider response codes.
    ///
    /// # Errors
    ///
    /// Returns an `Err` when the HTTP response status is not 200, or when the provider
    /// responds with an error code. Common error messages include:
    /// - `"HTTP error: <status_code>"` when the request returned a non-200 status.
    /// - `"Bad authorization (username or password)"` when credentials are rejected.
    /// - `"A Fully-Qualified Domain Name was not provided"` when `hostname` is not FQDN.
    /// - `"Hostname does not exist in the database"` when the host is unknown.
    /// - `"Hostname exists but not under this username"` when the host belongs to another account.
    /// - `"Hostname blocked for abuse"` when the host is blocked.
    /// - `"Unexpected response: <body>"` for any other provider response.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::net::IpAddr;
    /// # use std::str::FromStr;
    /// # // Assume `client` is a valid InwxClient configured with credentials.
    /// # let client = crate::clients::inwx::InwxClient::new(&crate::config::Config::default()).unwrap();
    /// let ip = IpAddr::from_str("203.0.113.42").unwrap();
    /// let _ = client.update_record("example.example", ip);
    /// ```
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let auth = general_purpose::STANDARD.encode(format!("{}:{}", self.username, self.password));
        
        let url = format!("{}/nic/update?hostname={}&myip={}", 
            self.server, hostname, ip);
        
        log::info!("Updating {} to {}", hostname, ip);
        
        let response = minreq::get(&url)
            .with_header("Authorization", format!("Basic {}", auth))
            .with_header("User-Agent", crate::USER_AGENT)
            .send()?;

        if response.status_code != 200 {
            return Err(format!("HTTP error: {}", response.status_code).into());
        }

        let body = response.as_str()?;
        
        // DynDNS2 response codes
        if body.contains("good") || body.contains("nochg") {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else if body.contains("badauth") {
            Err("Bad authorization (username or password)".into())
        } else if body.contains("notfqdn") {
            Err("A Fully-Qualified Domain Name was not provided".into())
        } else if body.contains("nohost") {
            Err("Hostname does not exist in the database".into())
        } else if body.contains("!yours") {
            Err("Hostname exists but not under this username".into())
        } else if body.contains("abuse") {
            Err("Hostname blocked for abuse".into())
        } else {
            Err(format!("Unexpected response: {}", body).into())
        }
    }

    /// Validates that the client has non-empty INWX credentials.
    ///
    /// Ensures both the stored username and password are not empty and returns an error describing
    /// which credential is missing when validation fails.
    ///
    /// # Returns
    ///
    /// `Ok(())` if both username and password are non-empty, otherwise an `Err` with a descriptive message.
    ///
    /// # Examples
    ///
    /// ```
    /// // Constructing directly for the example; adapt to your constructor in real code.
    /// let client = InwxClient {
    ///     username: "user".into(),
    ///     password: "pass".into(),
    ///     server: "https://dyndns.inwx.com".into(),
    /// };
    /// assert!(client.validate_config().is_ok());
    ///
    /// let bad = InwxClient {
    ///     username: "".into(),
    ///     password: "pass".into(),
    ///     server: "https://dyndns.inwx.com".into(),
    /// };
    /// assert!(client.validate_config().is_err());
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.username.is_empty() {
            return Err("INWX username cannot be empty".into());
        }
        if self.password.is_empty() {
            return Err("INWX password cannot be empty".into())
        }
        Ok(())
    }

    /// Provider identifier for this client.
    ///
    /// # Returns
    ///
    /// The provider name `"INWX"`.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = InwxClient { username: String::new(), password: String::new(), server: String::from("https://dyndns.inwx.com") };
    /// assert_eq!(client.provider_name(), "INWX");
    /// ```
    fn provider_name(&self) -> &str {
        "INWX"
    }
}