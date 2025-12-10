use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

/// OVH DynHost client
/// Uses OVH's DynDNS2-compatible DynHost service
/// API documentation: https://docs.ovh.com/gb/en/domains/hosting_dynhost/
pub struct OvhClient {
    server: String,
    login: String,
    password: String,
}

impl OvhClient {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let login = config.login.as_ref()
            .ok_or("login (DynHost username) is required for OVH")?
            .clone();
        
        let password = config.password.as_ref()
            .ok_or("password is required for OVH")?
            .clone();
        
        let server = config.server.clone()
            .unwrap_or_else(|| "www.ovh.com".to_string());

        Ok(OvhClient {
            server,
            login,
            password,
        })
    }
}

impl DnsClient for OvhClient {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        log::info!("Updating {} with OVH DynHost", hostname);

        // OVH uses DynDNS2-compatible protocol
        let url = format!(
            "https://{}/nic/update?system=dyndns&hostname={}&myip={}",
            self.server, hostname, ip
        );

        let auth = format!("{}:{}", self.login, self.password);
        use base64::{Engine as _, engine::general_purpose};
        let encoded_auth = general_purpose::STANDARD.encode(auth.as_bytes());

        let response = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .with_header("Authorization", format!("Basic {}", encoded_auth))
            .send()?;

        let status_code = response.status_code;
        let body = response.as_str()?.trim();

        log::debug!("Response status: {}, body: {}", status_code, body);

        if status_code != 200 {
            return Err(format!("HTTP {} error", status_code).into());
        }

        // Check response for success indicators (DynDNS2 protocol)
        if body.contains("good") {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else if body.contains("nochg") {
            log::info!("IP address for {} already set to {}", hostname, ip);
            Ok(())
        } else {
            Err(format!("OVH DynHost error: {}", body).into())
        }
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.login.is_empty() {
            return Err("login (DynHost username) is required for OVH".into());
        }
        if self.password.is_empty() {
            return Err("password is required for OVH".into());
        }
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "OVH"
    }
}
