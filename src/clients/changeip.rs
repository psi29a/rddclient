use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;
use base64::{Engine as _, engine::general_purpose};

pub struct ChangeipClient {
    username: String,
    password: String,
    server: String,
}

impl ChangeipClient {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let username = config.login.as_ref()
            .ok_or("username is required for ChangeIP")?;
        let password = config.password.as_ref()
            .ok_or("password is required for ChangeIP")?;
        let server = config.server.as_deref()
            .unwrap_or("nic.changeip.com");

        Ok(ChangeipClient {
            username: username.to_string(),
            password: password.to_string(),
            server: server.to_string(),
        })
    }
}

impl DnsClient for ChangeipClient {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        log::info!("Updating ChangeIP record for {} to {}", hostname, ip);

        let url = format!(
            "https://{}/nic/update?hostname={}&myip={}",
            self.server, hostname, ip
        );

        let auth = general_purpose::STANDARD.encode(format!("{}:{}", self.username, self.password));
        let response = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .with_header("Authorization", format!("Basic {}", auth))
            .send()?;

        if response.status_code != 200 {
            return Err(format!("ChangeIP API error: HTTP {}", response.status_code).into());
        }

        let body = response.as_str()?;
        
        // ChangeIP returns JSON response
        if body.contains("\"ok\":true") || body.contains("\"msg\":\"unaltered\"") {
            if body.contains("unaltered") {
                log::info!("IP address already set to {}", ip);
            } else {
                log::info!("Successfully updated DNS record for {} to {}", hostname, ip);
            }
            Ok(())
        } else if body.contains("\"ok\":false") {
            let error_msg = body
                .split("\"msg\":\"")
                .nth(1)
                .and_then(|s| s.split("\"").next())
                .unwrap_or("Unknown error");
            Err(format!("ChangeIP error: {}", error_msg).into())
        } else {
            Err(format!("Unexpected ChangeIP response: {}", body).into())
        }
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.username.is_empty() {
            return Err("ChangeIP username cannot be empty".into());
        }
        if self.password.is_empty() {
            return Err("ChangeIP password cannot be empty".into());
        }
        Ok(())
    }

    fn provider_name(&self) -> &'static str {
        "ChangeIP"
    }
}
