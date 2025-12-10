use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

/// LuaDNS client
/// Uses LuaDNS REST API
pub struct LuadnsClient {
    server: String,
    email: String,
    token: String,
    zone_id: String,
    record_id: String,
}

impl LuadnsClient {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let email = config.login.as_ref()
            .ok_or("username (email) is required for LuaDNS")?
            .clone();
        
        let token = config.api_token.as_ref()
            .ok_or("api_token is required for LuaDNS")?
            .clone();
        
        let zone_id = config.zone_id.as_ref()
            .ok_or("zone_id is required for LuaDNS")?
            .clone();
        
        let record_id = config.host.as_ref()
            .ok_or("dns_record (record ID) is required for LuaDNS")?
            .clone();
        
        let server = config.server.clone()
            .unwrap_or_else(|| "https://api.luadns.com".to_string());

        Ok(LuadnsClient {
            server,
            email,
            token,
            zone_id,
            record_id,
        })
    }
}

impl DnsClient for LuadnsClient {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let record_type = match ip {
            IpAddr::V4(_) => "A",
            IpAddr::V6(_) => "AAAA",
        };

        log::info!("Updating {} with LuaDNS", hostname);

        // LuaDNS API endpoint
        let url = format!("{}/v1/zones/{}/records/{}", 
            self.server, self.zone_id, self.record_id);

        let body = format!(
            r#"{{"content":"{}","type":"{}"}}"#,
            ip,
            record_type
        );

        let auth = format!("{}:{}", self.email, self.token);
        let encoded_auth = format!("Basic {}", base64::encode(&auth));

        let response = minreq::put(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .with_header("Authorization", &encoded_auth)
            .with_header("Content-Type", "application/json")
            .with_header("Accept", "application/json")
            .with_body(body)
            .send()?;

        let status_code = response.status_code;
        let response_body = response.as_str()?.trim();

        log::debug!("Response status: {}, body: {}", status_code, response_body);

        if status_code == 200 {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else if response_body.contains("error") {
            Err(format!("LuaDNS API error: {}", response_body).into())
        } else {
            Err(format!("HTTP error: {}", status_code).into())
        }
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.email.is_empty() {
            return Err("username (email) is required for LuaDNS".into());
        }
        if self.token.is_empty() {
            return Err("api_token is required for LuaDNS".into());
        }
        if self.zone_id.is_empty() {
            return Err("zone_id is required for LuaDNS".into());
        }
        if self.record_id.is_empty() {
            return Err("dns_record (record ID) is required for LuaDNS".into());
        }
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "LuaDNS"
    }
}

mod base64 {
    use base64::{Engine as _, engine::general_purpose};
    
    pub fn encode(data: &str) -> String {
        general_purpose::STANDARD.encode(data.as_bytes())
    }
}
