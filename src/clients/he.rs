use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

/// Hurricane Electric (HE.net) client - https://dns.he.net/
pub struct HurricaneElectricClient {
    password: String,
    server: String,
}

impl HurricaneElectricClient {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let password = config.password.as_ref()
            .ok_or("password is required for Hurricane Electric")?
            .clone();
        
        let server = config.server.as_ref()
            .cloned()
            .unwrap_or_else(|| "https://dyn.dns.he.net/nic/update".to_string());

        Ok(HurricaneElectricClient { password, server })
    }
}

impl DnsClient for HurricaneElectricClient {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let url = format!(
            "{}?hostname={}&password={}&myip={}",
            self.server, hostname, self.password, ip
        );

        log::info!("Updating {} with Hurricane Electric", hostname);

        let response = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .send()?;

        let body = response.as_str()?.trim();

        // HE.net returns various responses
        if body.contains("good") || body.contains("nochg") {
            log::info!("DNS record for {} successfully updated to {}", hostname, ip);
            Ok(())
        } else if body.contains("badauth") {
            Err("Bad authentication - check your password".into())
        } else if body.contains("notfqdn") {
            Err("Not a fully-qualified domain name".into())
        } else {
            Err(format!("Unexpected response: {}", body).into())
        }
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.password.is_empty() {
            return Err("password is required for Hurricane Electric".into());
        }
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "Hurricane Electric"
    }
}
