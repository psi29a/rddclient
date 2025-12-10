use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

/// Namecheap Dynamic DNS client
pub struct NamecheapClient {
    server: String,
    domain: String,
    password: String,
}

impl NamecheapClient {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        // For Namecheap, username is the domain name
        let domain = config.login.as_ref()
            .ok_or("username (domain) is required for Namecheap")?
            .clone();
        let password = config.password.as_ref()
            .ok_or("password is required for Namecheap")?
            .clone();
        
        let server = config.server.as_ref()
            .cloned()
            .unwrap_or_else(|| "dynamicdns.park-your-domain.com".to_string());

        Ok(NamecheapClient {
            server,
            domain,
            password,
        })
    }
}

impl DnsClient for NamecheapClient {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        // Extract subdomain from hostname
        // e.g., "www.example.com" with domain "example.com" -> "www"
        let host = if hostname.ends_with(&format!(".{}", self.domain)) {
            hostname.trim_end_matches(&format!(".{}", self.domain))
        } else if hostname == self.domain {
            "@"
        } else {
            hostname
        };

        let url = format!(
            "https://{}/update?host={}&domain={}&password={}&ip={}",
            self.server, host, self.domain, self.password, ip
        );

        log::info!("Updating {} with Namecheap", hostname);

        let response = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .send()?;

        let status_code = response.status_code;
        let body = response.as_str()?;

        log::debug!("Response status: {}, body: {}", status_code, body);

        if status_code != 200 {
            return Err(format!("HTTP error: {}", status_code).into());
        }

        // Namecheap returns XML response
        // Success: <ErrCount>0</ErrCount>
        // Failure: <ErrCount>1</ErrCount> (or higher)
        if body.contains("<ErrCount>0") {
            log::info!("DNS record for {} successfully updated to {}", hostname, ip);
            Ok(())
        } else {
            // Try to extract error message
            if let Some(start) = body.find("<Err1>") {
                if let Some(end) = body[start..].find("</Err1>") {
                    let error_msg = &body[start + 6..start + end];
                    return Err(format!("Namecheap error: {}", error_msg).into());
                }
            }
            Err(format!("Update failed: {}", body).into())
        }
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.domain.is_empty() {
            return Err("username (domain) is required for Namecheap".into());
        }
        if self.password.is_empty() {
            return Err("password is required for Namecheap".into());
        }
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "Namecheap"
    }
}
