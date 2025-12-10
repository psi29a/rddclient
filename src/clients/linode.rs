use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

/// Linode DNS client
/// Uses Linode API v4
pub struct LinodeClient {
    server: String,
    token: String,
    domain_id: String,
    record_id: String,
}

impl LinodeClient {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let token = config.password.as_ref()
            .ok_or("api_token is required for Linode")?
            .clone();
        
        let domain_id = config.zone.as_ref()
            .ok_or("zone_id (domain ID) is required for Linode")?
            .clone();
        
        let record_id = config.host.as_ref()
            .ok_or("dns_record (record ID) is required for Linode")?
            .clone();
        
        let server = config.server.clone()
            .unwrap_or_else(|| "https://api.linode.com".to_string());

        Ok(LinodeClient {
            server,
            token,
            domain_id,
            record_id,
        })
    }
}

impl DnsClient for LinodeClient {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let record_type = match ip {
            IpAddr::V4(_) => "A",
            IpAddr::V6(_) => "AAAA",
        };

        log::info!("Updating {} with Linode", hostname);

        // Linode API v4 endpoint
        let url = format!("{}/v4/domains/{}/records/{}", 
            self.server, self.domain_id, self.record_id);

        let body = format!(
            r#"{{"type":"{}","target":"{}"}}"#,
            record_type,
            ip
        );

        let response = minreq::put(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .with_header("Authorization", &format!("Bearer {}", self.token))
            .with_header("Content-Type", "application/json")
            .with_body(body)
            .send()?;

        let status_code = response.status_code;
        let response_body = response.as_str()?.trim();

        log::debug!("Response status: {}, body: {}", status_code, response_body);

        if status_code == 200 {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else if response_body.contains("errors") {
            Err(format!("Linode API error: {}", response_body).into())
        } else {
            Err(format!("HTTP error: {}", status_code).into())
        }
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.token.is_empty() {
            return Err("api_token is required for Linode".into());
        }
        if self.domain_id.is_empty() {
            return Err("zone_id (domain ID) is required for Linode".into());
        }
        if self.record_id.is_empty() {
            return Err("dns_record (record ID) is required for Linode".into());
        }
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "Linode"
    }
}
