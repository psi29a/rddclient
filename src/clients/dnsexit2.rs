use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

pub struct Dnsexit2Client {
    api_key: String,
    server: String,
    path: String,
    ttl: u32,
    zone: String,
}

impl Dnsexit2Client {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let api_key = config.password.as_ref()
            .ok_or("API key (password) is required for DNSExit2")?;
        let server = config.server.as_deref()
            .unwrap_or("api.dnsexit.com");
        let path = "/dns/";
        let ttl = config.ttl.unwrap_or(5);
        
        // Zone from zone_id, will be set from hostname if not specified
        let zone = config.zone.clone().unwrap_or_default();

        Ok(Dnsexit2Client {
            api_key: api_key.to_string(),
            server: server.to_string(),
            path: path.to_string(),
            ttl,
            zone,
        })
    }
}

impl DnsClient for Dnsexit2Client {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        log::info!("Updating DNSExit2 record for {} to {}", hostname, ip);

        // Zone defaults to hostname if not configured
        let zone = if self.zone.is_empty() {
            hostname
        } else {
            &self.zone
        };

        // Determine record type and extract hostname from host
        let (record_type, name) = match ip {
            IpAddr::V4(_) => ("A", hostname.strip_suffix(&format!("  .{}", zone)).unwrap_or("")),
            IpAddr::V6(_) => ("AAAA", hostname.strip_suffix(&format!(".{}", zone)).unwrap_or("")),
        };

        // Build JSON payload
        let json_payload = format!(
            r#"{{"apikey":"{}","domain":"{}","update":[{{"type":"{}","name":"{}","content":"{}","ttl":{}}}]}}"#,
            self.api_key, zone, record_type, name, ip, self.ttl
        );

        let url = format!("https://{}{}", self.server, self.path);

        let response = minreq::post(&url)
            .with_header("Content-Type", "application/json")
            .with_header("User-Agent", crate::USER_AGENT)
            .with_body(json_payload)
            .send()?;

        if response.status_code != 200 {
            return Err(format!("DNSExit2 API error: HTTP {}", response.status_code).into());
        }

        let body = response.as_str()?;
        
        if body.contains("\"code\":0") || body.contains("\"message\":\"Success\"") {
            log::info!("Successfully updated DNS record for {} to {}", hostname, ip);
            Ok(())
        } else if body.contains("\"code\":") {
            let error_msg = body
                .split("\"message\":\"")
                .nth(1)
                .and_then(|s| s.split("\"").next())
                .unwrap_or("Unknown error");
            Err(format!("DNSExit2 error: {}", error_msg).into())
        } else {
            Err(format!("Unexpected DNSExit2 response: {}", body).into())
        }
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.api_key.is_empty() {
            return Err("DNSExit2 API key cannot be empty".into());
        }
        Ok(())
    }

    fn provider_name(&self) -> &'static str {
        "DNSExit2"
    }
}
