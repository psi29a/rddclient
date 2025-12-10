use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;
use base64::{Engine as _, engine::general_purpose};

/// No-IP client - compatible with DynDNS2 but with No-IP specifics
pub struct NoIpClient {
    username: String,
    password: String,
    server: String,
}

impl NoIpClient {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let username = config.login.as_ref()
            .ok_or("username is required for No-IP")?
            .clone();
        let password = config.password.as_ref()
            .ok_or("password is required for No-IP")?
            .clone();
        
        let server = config.server.as_ref()
            .cloned()
            .unwrap_or_else(|| "https://dynupdate.no-ip.com".to_string());

        Ok(NoIpClient {
            username,
            password,
            server,
        })
    }
}

impl DnsClient for NoIpClient {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let url = format!(
            "{}/nic/update?hostname={}&myip={}",
            self.server, hostname, ip
        );

        log::info!("Updating {} with No-IP", hostname);

        let response = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .with_header("Authorization", format!("Basic {}", 
                general_purpose::STANDARD.encode(format!("{}:{}", self.username, self.password))))
            .send()?;

        let body = response.as_str()?.trim();
        let status = body.split_whitespace().next().unwrap_or("");

        match status {
            "good" | "nochg" => {
                log::info!("DNS record for {} successfully updated to {}", hostname, ip);
                Ok(())
            }
            "badauth" => Err("Bad authentication".into()),
            "nohost" => Err("Hostname doesn't exist".into()),
            "badagent" => Err("Client disabled - contact No-IP".into()),
            "abuse" => Err("Username blocked for abuse".into()),
            "911" => Err("Server error - try again later".into()),
            _ => Err(format!("Unknown response: {}", body).into()),
        }
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.username.is_empty() {
            return Err("username is required for No-IP".into());
        }
        if self.password.is_empty() {
            return Err("password is required for No-IP".into());
        }
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "No-IP"
    }
}
