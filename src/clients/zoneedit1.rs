use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;
use base64::{Engine as _, engine::general_purpose};

/// ZoneEdit v1 DNS client (legacy protocol)
/// Uses ZoneEdit's legacy dynamic DNS protocol
pub struct Zoneedit1Client {
    server: String,
    username: String,
    password: String,
}

impl Zoneedit1Client {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let username = config.login.as_ref()
            .ok_or("username is required for ZoneEdit v1")?
            .clone();
        
        let password = config.password.as_ref()
            .ok_or("password is required for ZoneEdit v1")?
            .clone();
        
        let server = config.server.clone()
            .unwrap_or_else(|| "https://dynamic.zoneedit.com".to_string());

        Ok(Zoneedit1Client {
            server,
            username,
            password,
        })
    }
}

impl DnsClient for Zoneedit1Client {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        log::info!("Updating {} with ZoneEdit v1", hostname);

        // ZoneEdit v1 update endpoint
        let url = format!("{}/auth/dynamic.html", self.server);

        let auth = format!("{}:{}", self.username, self.password);
        let encoded_auth = format!("Basic {}", general_purpose::STANDARD.encode(auth.as_bytes()));

        let response = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .with_header("Authorization", &encoded_auth)
            .with_param("host", hostname)
            .with_param("dnsto", ip.to_string())
            .send()?;

        let status_code = response.status_code;
        let body = response.as_str()?.trim();

        log::debug!("Response status: {}, body: {}", status_code, body);

        if status_code != 200 {
            return Err(format!("HTTP error: {}", status_code).into());
        }

        // ZoneEdit v1 returns HTML with status indicators
        if body.contains("SUCCESS") || body.contains("UPDATE") {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else if body.contains("ERROR CODE=\"707\"") {
            Err("Update failed - duplicate update".into())
        } else if body.contains("ERROR CODE=\"701\"") {
            Err("Zone not found".into())
        } else if body.contains("ERROR CODE=\"702\"") {
            Err("Record not found".into())
        } else if body.contains("ERROR") {
            Err(format!("ZoneEdit v1 error: {}", body).into())
        } else if body.contains("INVALID_USER") || body.contains("INVALID_PASS") {
            Err("Authentication failed".into())
        } else {
            Err(format!("Unexpected response: {}", body).into())
        }
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.username.is_empty() {
            return Err("username is required for ZoneEdit v1".into());
        }
        if self.password.is_empty() {
            return Err("password is required for ZoneEdit v1".into());
        }
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "ZoneEdit v1"
    }
}
