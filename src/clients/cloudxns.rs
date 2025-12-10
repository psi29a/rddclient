use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

/// CloudXNS DNS client
/// Uses CloudXNS REST API
pub struct CloudXnsClient {
    server: String,
    api_key: String,
    secret_key: String,
}

impl CloudXnsClient {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let api_key = config.login.as_ref()
            .ok_or("username (API key) is required for CloudXNS")?
            .clone();
        let secret_key = config.password.as_ref()
            .ok_or("password (secret key) is required for CloudXNS")?
            .clone();
        
        let server = config.server.clone()
            .unwrap_or_else(|| "https://www.cloudxns.net".to_string());

        Ok(CloudXnsClient {
            server,
            api_key,
            secret_key,
        })
    }
}

impl DnsClient for CloudXnsClient {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let record_type = match ip {
            IpAddr::V4(_) => "A",
            IpAddr::V6(_) => "AAAA",
        };

        // CloudXNS API endpoint
        let url = format!("{}/api2/ddns", self.server);

        log::info!("Updating {} with CloudXNS", hostname);

        // CloudXNS uses custom authentication headers
        let response = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .with_header("API-KEY", &self.api_key)
            .with_header("API-REQUEST-DATE", &format!("{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()))
            .with_param("domain", hostname)
            .with_param("ip", &ip.to_string())
            .with_param("type", record_type)
            .send()?;

        let status_code = response.status_code;
        let body = response.as_str()?.trim();

        log::debug!("Response status: {}, body: {}", status_code, body);

        if status_code != 200 {
            return Err(format!("HTTP error: {}", status_code).into());
        }

        // Parse JSON response
        if body.contains(r#""code":1"#) || body.contains("success") {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else if body.contains("error") {
            Err(format!("CloudXNS error: {}", body).into())
        } else {
            Err(format!("Unexpected response: {}", body).into())
        }
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.api_key.is_empty() {
            return Err("username (API key) is required for CloudXNS".into());
        }
        if self.secret_key.is_empty() {
            return Err("password (secret key) is required for CloudXNS".into());
        }
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "CloudXNS"
    }
}
