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

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.username.is_empty() {
            return Err("INWX username cannot be empty".into());
        }
        if self.password.is_empty() {
            return Err("INWX password cannot be empty".into())
        }
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "INWX"
    }
}
