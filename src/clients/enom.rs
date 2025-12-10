use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

/// Enom DNS client
/// Uses Enom's Dynamic DNS API
pub struct EnomClient {
    server: String,
    password: String,
}

impl EnomClient {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let password = config.password.as_ref()
            .ok_or("password (update token) is required for Enom")?
            .clone();
        
        let server = config.server.as_ref()
            .cloned()
            .unwrap_or_else(|| "https://dynamic.name-services.com".to_string());

        Ok(EnomClient {
            server,
            password,
        })
    }
}

impl DnsClient for EnomClient {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let url = format!(
            "{}/interface.asp?command=SetDNSHost&HostName={}&Zone={}&Address={}&DomainPassword={}",
            self.server,
            hostname.split('.').next().unwrap_or(""),
            hostname.split('.').skip(1).collect::<Vec<_>>().join("."),
            ip,
            self.password
        );

        log::info!("Updating {} with Enom", hostname);

        let response = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .send()?;

        let status_code = response.status_code;
        let body = response.as_str()?.trim();

        log::debug!("Response status: {}, body: {}", status_code, body);

        if status_code != 200 {
            return Err(format!("HTTP error: {}", status_code).into());
        }

        // Parse Enom response
        if body.contains("ErrCount=0") || body.contains("success") {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else if body.contains("ErrCount=") {
            Err(format!("Enom error: {}", body).into())
        } else {
            Err(format!("Unexpected response: {}", body).into())
        }
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.password.is_empty() {
            return Err("password (update token) is required for Enom".into());
        }
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "Enom"
    }
}
