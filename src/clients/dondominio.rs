use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

/// DonDominio DNS client
/// Uses DonDominio's dondns API with API key authentication
pub struct DonDominioClient {
    server: String,
    api_key: String,
    username: String,
}

impl DonDominioClient {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let api_key = config.password.as_ref()
            .ok_or("password (API key) is required for DonDominio")?
            .clone();
        let username = config.login.as_ref()
            .ok_or("username is required for DonDominio")?
            .clone();
        
        let server = config.server.as_ref()
            .cloned()
            .unwrap_or_else(|| "https://dondns.dondominio.com".to_string());

        Ok(DonDominioClient {
            server,
            api_key,
            username,
        })
    }
}

impl DnsClient for DonDominioClient {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let record_type = match ip {
            IpAddr::V4(_) => "A",
            IpAddr::V6(_) => "AAAA",
        };

        let url = format!("{}/update", self.server);
        
        let json_body = format!(
            r#"{{"apiuser":"{}","apipasswd":"{}","domain":"{}","name":"{}","type":"{}","value":"{}"}}"#,
            self.username,
            self.api_key,
            hostname.split('.').skip(1).collect::<Vec<_>>().join("."),
            hostname.split('.').next().unwrap_or(""),
            record_type,
            ip
        );

        log::info!("Updating {} with DonDominio", hostname);

        let response = minreq::post(&url)
            .with_header("Content-Type", "application/json")
            .with_header("User-Agent", crate::USER_AGENT)
            .with_body(json_body)
            .send()?;

        let status_code = response.status_code;
        let body = response.as_str()?.trim();

        log::debug!("Response status: {}, body: {}", status_code, body);

        if status_code != 200 {
            return Err(format!("HTTP error: {}", status_code).into());
        }

        // Parse JSON response
        if body.contains(r#""success":true"#) || body.contains(r#""success":"true""#) {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else if body.contains("authentication") || body.contains("credentials") {
            Err("Authentication failed - check username and API key".into())
        } else if body.contains("error") {
            Err(format!("DonDominio error: {}", body).into())
        } else {
            Err(format!("Unexpected response: {}", body).into())
        }
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.username.is_empty() {
            return Err("username is required for DonDominio".into());
        }
        if self.api_key.is_empty() {
            return Err("password (API key) is required for DonDominio".into());
        }
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "DonDominio"
    }
}
