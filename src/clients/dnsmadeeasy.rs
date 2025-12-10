use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

/// DNS Made Easy client
/// Uses simplified API (full REST API with HMAC would be more complex)
pub struct DnsMadeEasyClient {
    server: String,
    username: String,
    password: String,
}

impl DnsMadeEasyClient {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let username = config.login.as_ref()
            .ok_or("username is required for DNS Made Easy")?
            .clone();
        let password = config.password.as_ref()
            .ok_or("password is required for DNS Made Easy")?
            .clone();
        
        let server = config.server.as_ref()
            .cloned()
            .unwrap_or_else(|| "https://cp.dnsmadeeasy.com".to_string());

        Ok(DnsMadeEasyClient {
            server,
            username,
            password,
        })
    }
}

impl DnsClient for DnsMadeEasyClient {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        // DNS Made Easy dynamic DNS endpoint
        let url = format!(
            "{}/servlet/updateip?username={}&password={}&id={}&ip={}",
            self.server, self.username, self.password, hostname, ip
        );

        log::info!("Updating {} with DNS Made Easy", hostname);

        let response = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .send()?;

        let status_code = response.status_code;
        let body = response.as_str()?.trim();

        log::debug!("Response status: {}, body: {}", status_code, body);

        if status_code != 200 {
            return Err(format!("HTTP error: {}", status_code).into());
        }

        // Parse response
        if body.contains("success") || body.contains("updated") || body == "good" {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else if body.contains("error") || body.contains("invalid") {
            Err(format!("DNS Made Easy error: {}", body).into())
        } else {
            Err(format!("Unexpected response: {}", body).into())
        }
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.username.is_empty() {
            return Err("username is required for DNS Made Easy".into());
        }
        if self.password.is_empty() {
            return Err("password is required for DNS Made Easy".into());
        }
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "DNS Made Easy"
    }
}
