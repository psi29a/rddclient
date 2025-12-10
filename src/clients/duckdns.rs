use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

/// DuckDNS client - https://www.duckdns.org/
pub struct DuckDnsClient {
    token: String,
    server: String,
}

impl DuckDnsClient {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let token = config.password.as_ref()
            .or(config.api_token.as_ref())
            .ok_or("token (password or api_token) is required for DuckDNS")?
            .clone();
        
        let server = config.server.as_ref()
            .cloned()
            .unwrap_or_else(|| "https://www.duckdns.org".to_string());

        Ok(DuckDnsClient { token, server })
    }
}

impl DnsClient for DuckDnsClient {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        // DuckDNS hostname is typically without the .duckdns.org suffix
        let domain = hostname.trim_end_matches(".duckdns.org");
        
        let url = format!(
            "{}/update?domains={}&token={}&ip={}",
            self.server, domain, self.token, ip
        );

        log::info!("Updating {} with DuckDNS", hostname);

        let response = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .send()?;

        let body = response.as_str()?.trim();

        if body == "OK" {
            log::info!("DNS record for {} successfully updated to {}", hostname, ip);
            Ok(())
        } else if body == "KO" {
            Err("DuckDNS update failed - check your token and domain".into())
        } else {
            Err(format!("Unexpected response: {}", body).into())
        }
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.token.is_empty() {
            return Err("token is required for DuckDNS".into());
        }
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "DuckDNS"
    }
}
