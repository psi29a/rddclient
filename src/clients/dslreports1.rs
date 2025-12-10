use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

/// DSLReports DNS client (legacy v1 protocol)
/// Uses DSLReports legacy update protocol
pub struct Dslreports1Client {
    server: String,
    username: String,
    password: String,
}

impl Dslreports1Client {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let username = config.login.as_ref()
            .ok_or("username is required for DSLReports")?
            .clone();
        
        let password = config.password.as_ref()
            .ok_or("password is required for DSLReports")?
            .clone();
        
        let server = config.server.clone()
            .unwrap_or_else(|| "https://www.dslreports.com".to_string());

        Ok(Dslreports1Client {
            server,
            username,
            password,
        })
    }
}

impl DnsClient for Dslreports1Client {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        log::info!("Updating {} with DSLReports", hostname);

        // DSLReports legacy update endpoint
        let url = format!("{}/updateip", self.server);

        let response = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .with_param("username", &self.username)
            .with_param("password", &self.password)
            .with_param("hostname", hostname)
            .with_param("ip", &ip.to_string())
            .send()?;

        let status_code = response.status_code;
        let body = response.as_str()?.trim();

        log::debug!("Response status: {}, body: {}", status_code, body);

        if status_code != 200 {
            return Err(format!("HTTP error: {}", status_code).into());
        }

        // DSLReports returns simple text responses
        if body.contains("good") || body.contains("successful") || body == "OK" {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else if body.contains("badauth") {
            Err("Authentication failed".into())
        } else if body.contains("notfqdn") {
            Err("Invalid hostname".into())
        } else {
            Err(format!("Update failed: {}", body).into())
        }
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.username.is_empty() {
            return Err("username is required for DSLReports".into());
        }
        if self.password.is_empty() {
            return Err("password is required for DSLReports".into());
        }
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "DSLReports"
    }
}
