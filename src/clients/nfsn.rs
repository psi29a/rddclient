use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

/// NearlyFreeSpeech.NET (NFSN) DNS client
/// Uses NFSN dynamic DNS API
pub struct NfsnClient {
    server: String,
    username: String,
    password: String,
}

impl NfsnClient {
    /// Constructs a new NfsnClient from the given configuration.
    ///
    /// The function reads `login` and `password` from `config` (both required) and uses
    /// `config.server` if provided, otherwise defaults to
    /// "https://dynamicdns.park-your-domain.com".
    ///
    /// # Parameters
    ///
    /// - `config`: configuration containing `login`, `password`, and optional `server`.
    ///
    /// # Errors
    ///
    /// Returns an error if `login` (username) or `password` is missing from `config`.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut cfg = Config::default();
    /// cfg.login = Some("user".to_string());
    /// cfg.password = Some("secret".to_string());
    /// let client = NfsnClient::new(&cfg).unwrap();
    /// assert_eq!(client.provider_name(), "NFSN");
    /// ```
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let username = config.login.as_ref()
            .ok_or("username is required for NFSN")?
            .clone();
        
        let password = config.password.as_ref()
            .ok_or("password is required for NFSN")?
            .clone();
        
        let server = config.server.clone()
            .unwrap_or_else(|| "https://dynamicdns.park-your-domain.com".to_string());

        Ok(NfsnClient {
            server,
            username,
            password,
        })
    }
}

impl DnsClient for NfsnClient {
    /// Update the DNS A record for `hostname` to the given `ip` using the NFSN dynamic DNS API.
    ///
    /// Sends a Namecheap-compatible HTTP GET to the provider's `/update` endpoint using HTTP
    /// Basic authentication. Returns `Ok(())` when the provider indicates success; returns
    /// `Err` when the HTTP response status is not 200, when the provider reports an error
    /// in the response body, or when a transport/parsing error occurs.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::IpAddr;
    /// // Construct a client with the NFSN API base URL, username, and password.
    /// let client = NfsnClient {
    ///     server: "https://dynamicdns.park-your-domain.com".into(),
    ///     username: "user".into(),
    ///     password: "pass".into(),
    /// };
    /// let ip: IpAddr = "203.0.113.42".parse().unwrap();
    /// client.update_record("example.example", ip).expect("update failed");
    /// ```
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        log::info!("Updating {} with NFSN", hostname);

        // NFSN uses a Namecheap-compatible endpoint
        let url = format!("{}/update", self.server);

        let auth = format!("{}:{}", self.username, self.password);
        let encoded_auth = format!("Basic {}", base64::encode(&auth));

        let response = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .with_header("Authorization", &encoded_auth)
            .with_param("host", hostname)
            .with_param("ip", ip.to_string())
            .send()?;

        let status_code = response.status_code;
        let body = response.as_str()?.trim();

        log::debug!("Response status: {}, body: {}", status_code, body);

        if status_code != 200 {
            return Err(format!("HTTP error: {}", status_code).into());
        }

        // Check response for success indicators
        if body.contains("<ErrCount>0</ErrCount>") || body.contains("success") {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else if body.contains("<Err1>") {
            // Extract error message from XML
            Err(format!("NFSN error: {}", body).into())
        } else {
            Err(format!("Unexpected response: {}", body).into())
        }
    }

    /// Validate that the client has non-empty credentials.
    ///
    /// Returns an `Err` if `username` or `password` is an empty string; otherwise returns `Ok(())`.
    ///
    /// The returned error messages are:
    /// - `"username is required for NFSN"` when `username` is empty.
    /// - `"password is required for NFSN"` when `password` is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = NfsnClient {
    ///     server: String::from("https://dynamicdns.park-your-domain.com"),
    ///     username: String::from("user"),
    ///     password: String::from("pass"),
    /// };
    /// assert!(client.validate_config().is_ok());
    /// ```
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.username.is_empty() {
            return Err("username is required for NFSN".into());
        }
        if self.password.is_empty() {
            return Err("password is required for NFSN".into());
        }
        Ok(())
    }

    /// Provider name identifier for this client.
    ///
    /// This returns the static provider name `"NFSN"`.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = NfsnClient {
    ///     server: String::from("https://example"),
    ///     username: String::from("user"),
    ///     password: String::from("pass"),
    /// };
    /// assert_eq!(client.provider_name(), "NFSN");
    /// ```
    fn provider_name(&self) -> &str {
        "NFSN"
    }
}

mod base64 {
    use base64::{Engine as _, engine::general_purpose};
    
    /// Encode a UTF-8 string into Base64 using the standard character set.
    ///
    /// Returns the Base64 representation of `data`.
    ///
    /// # Examples
    ///
    /// ```
    /// let encoded = crate::clients::nfsn::base64::encode("user:password");
    /// assert_eq!(encoded, "dXNlcjpwYXNzd29yZA==");
    /// ```
    pub fn encode(data: &str) -> String {
        general_purpose::STANDARD.encode(data.as_bytes())
    }
}