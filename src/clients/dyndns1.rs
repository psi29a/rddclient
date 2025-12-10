use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;
use base64::{Engine as _, engine::general_purpose};

/// DynDNS v1 DNS client (legacy protocol)
/// Uses the original DynDNS v1 protocol (predates DynDNS2)
pub struct Dyndns1Client {
    server: String,
    username: String,
    password: String,
    static_ip: bool,
}

impl Dyndns1Client {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let username = config.login.as_ref()
            .ok_or("username is required for DynDNS v1")?
            .clone();
        
        let password = config.password.as_ref()
            .ok_or("password is required for DynDNS v1")?
            .clone();
        
        let server = config.server.clone()
            .unwrap_or_else(|| "https://members.dyndns.org".to_string());
        
        // Static IP flag for legacy DynDNS
        let static_ip = false;

        Ok(Dyndns1Client {
            server,
            username,
            password,
            static_ip,
        })
    }
}

impl DnsClient for Dyndns1Client {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        log::info!("Updating {} with DynDNS v1", hostname);

        // DynDNS v1 update endpoint
        let url = format!("{}/nic/update", self.server);

        let auth = format!("{}:{}", self.username, self.password);
        let encoded_auth = format!("Basic {}", general_purpose::STANDARD.encode(auth.as_bytes()));

        let mut request = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .with_header("Authorization", &encoded_auth)
            .with_param("hostname", hostname)
            .with_param("myip", &ip.to_string());

        // Add system parameter for static IPs (DynDNS v1 specific)
        if self.static_ip {
            request = request.with_param("system", "statdns");
        } else {
            request = request.with_param("system", "dyndns");
        }

        let response = request.send()?;

        let status_code = response.status_code;
        let body = response.as_str()?.trim();

        log::debug!("Response status: {}, body: {}", status_code, body);

        if status_code != 200 {
            return Err(format!("HTTP error: {}", status_code).into());
        }

        // DynDNS v1 protocol response codes
        if body.starts_with("good") || body.starts_with("nochg") {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else if body.starts_with("badauth") {
            Err("Authentication failed".into())
        } else if body.starts_with("notfqdn") {
            Err("Invalid hostname".into())
        } else if body.starts_with("nohost") {
            Err("Hostname not found".into())
        } else if body.starts_with("!donator") {
            Err("Feature requires donator account".into())
        } else if body.starts_with("!active") {
            Err("Hostname not activated".into())
        } else if body.starts_with("abuse") {
            Err("Hostname blocked for abuse".into())
        } else {
            Err(format!("Update failed: {}", body).into())
        }
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.username.is_empty() {
            return Err("username is required for DynDNS v1".into());
        }
        if self.password.is_empty() {
            return Err("password is required for DynDNS v1".into());
        }
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "DynDNS v1"
    }
}
