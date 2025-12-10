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
            .with_param("ip", &ip.to_string())
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

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.username.is_empty() {
            return Err("username is required for NFSN".into());
        }
        if self.password.is_empty() {
            return Err("password is required for NFSN".into());
        }
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "NFSN"
    }
}

mod base64 {
    use base64::{Engine as _, engine::general_purpose};
    
    pub fn encode(data: &str) -> String {
        general_purpose::STANDARD.encode(data.as_bytes())
    }
}
