use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

/// DNSPod DNS client
/// Uses DNSPod token-based API
pub struct DnspodClient {
    server: String,
    token: String,
}

impl DnspodClient {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let token = config.api_token.as_ref()
            .ok_or("api_token is required for DNSPod")?
            .clone();
        
        let server = config.server.clone()
            .unwrap_or_else(|| "https://dnsapi.cn".to_string());

        Ok(DnspodClient {
            server,
            token,
        })
    }
}

impl DnsClient for DnspodClient {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let record_type = match ip {
            IpAddr::V4(_) => "A",
            IpAddr::V6(_) => "AAAA",
        };

        // Split hostname into subdomain and domain
        let parts: Vec<&str> = hostname.rsplitn(3, '.').collect();
        if parts.len() < 2 {
            return Err("Invalid hostname format".into());
        }
        let domain = format!("{}.{}", parts[1], parts[0]);
        let subdomain = if parts.len() >= 3 {
            parts[2]
        } else {
            "@"
        };

        log::info!("Updating {} with DNSPod", hostname);

        // DNSPod API endpoint
        let url = format!("{}/Record.Ddns", self.server);

        let response = minreq::post(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .with_header("Content-Type", "application/x-www-form-urlencoded")
            .with_body(format!(
                "login_token={}&format=json&domain={}&sub_domain={}&record_type={}&value={}",
                self.token,
                domain,
                subdomain,
                record_type,
                ip
            ))
            .send()?;

        let status_code = response.status_code;
        let body = response.as_str()?.trim();

        log::debug!("Response status: {}, body: {}", status_code, body);

        if status_code != 200 {
            return Err(format!("HTTP error: {}", status_code).into());
        }

        // Parse JSON response
        if body.contains(r#""code":"1""#) || body.contains("success") {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else if body.contains(r#""message":"#) {
            // Extract error message
            Err(format!("DNSPod error: {}", body).into())
        } else {
            Err(format!("Unexpected response: {}", body).into())
        }
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.token.is_empty() {
            return Err("api_token is required for DNSPod".into());
        }
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "DNSPod"
    }
}
