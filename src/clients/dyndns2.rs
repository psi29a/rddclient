use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;
use base64::{Engine as _, engine::general_purpose};

/// DynDNS2 protocol client
/// This protocol is supported by many providers including:
/// - DynDNS
/// - No-IP
/// - DNSdynamic
/// - DuckDNS
/// - Many others
pub struct DynDns2Client {
    server: String,
    username: String,
    password: String,
    script: String,
}

impl DynDns2Client {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let username = config.login.as_ref()
            .ok_or("username is required for DynDNS2")?
            .clone();
        let password = config.password.as_ref()
            .ok_or("password is required for DynDNS2")?
            .clone();
        
        // Default server and script path for standard DynDNS2 protocol
        let server = config.server.as_ref()
            .cloned()
            .unwrap_or_else(|| "https://members.dyndns.org".to_string());
        let script = "/nic/update".to_string();

        Ok(DynDns2Client {
            server,
            username,
            password,
            script,
        })
    }
}

impl DnsClient for DynDns2Client {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let url = format!(
            "{}{}?hostname={}&myip={}",
            self.server, self.script, hostname, ip
        );

        log::info!("Updating {} with DynDNS2 protocol", hostname);

        let response = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .with_header("Authorization", format!("Basic {}", 
                general_purpose::STANDARD.encode(format!("{}:{}", self.username, self.password))))
            .send()?;

        let status_code = response.status_code;
        let body = response.as_str()?.trim();

        log::debug!("Response status: {}, body: {}", status_code, body);

        if status_code != 200 {
            return Err(format!("HTTP error: {}", status_code).into());
        }

        // Parse DynDNS2 response
        // Format: "status [ip]" where status is one of:
        // - good: Update successful
        // - nochg: No change needed (IP is the same)
        // - badauth: Bad authorization (username/password)
        // - notfqdn: Not a fully-qualified domain name
        // - nohost: Hostname doesn't exist
        // - !yours: Hostname exists but not under this account
        // - abuse: Hostname blocked for abuse
        
        let parts: Vec<&str> = body.split_whitespace().collect();
        let status = parts.first().ok_or("Empty response from server")?;

        match *status {
            "good" => {
                log::info!("DNS record for {} successfully updated to {}", hostname, ip);
                Ok(())
            }
            "nochg" => {
                log::info!("DNS record for {} already set to {} (no change)", hostname, ip);
                Ok(())
            }
            "badauth" => Err("Bad authorization (username or password)".into()),
            "notfqdn" => Err("Not a fully-qualified domain name".into()),
            "nohost" => Err("Hostname doesn't exist".into()),
            "!yours" => Err("Hostname exists but not under this account".into()),
            "abuse" => Err("Hostname blocked for abuse".into()),
            "!donator" => Err("Feature requires donator account".into()),
            "!active" => Err("Hostname not activated".into()),
            "dnserr" => Err("DNS error on server".into()),
            _ => Err(format!("Unknown response: {}", body).into()),
        }
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.username.is_empty() {
            return Err("username is required for DynDNS2".into());
        }
        if self.password.is_empty() {
            return Err("password is required for DynDNS2".into());
        }
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "DynDNS2"
    }
}
