use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

/// Yandex PDD (Yandex.Connect) DNS client
/// Uses Yandex PDD API with OAuth token
pub struct YandexClient {
    server: String,
    token: String,
    domain: String,
}

impl YandexClient {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let token = config.password.as_ref()
            .ok_or("password (PDD token) is required for Yandex")?
            .clone();
        let domain = config.zone_id.as_ref()
            .ok_or("zone_id (domain) is required for Yandex")?
            .clone();
        
        let server = config.server.as_ref()
            .cloned()
            .unwrap_or_else(|| "https://pddimp.yandex.ru".to_string());

        Ok(YandexClient {
            server,
            token,
            domain,
        })
    }
}

impl DnsClient for YandexClient {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        let record_type = match ip {
            IpAddr::V4(_) => "A",
            IpAddr::V6(_) => "AAAA",
        };

        // Extract subdomain from hostname
        let subdomain = if hostname.ends_with(&format!(".{}", self.domain)) {
            hostname.trim_end_matches(&format!(".{}", self.domain))
        } else {
            hostname
        };

        let url = format!(
            "{}/api2/admin/dns/edit?domain={}&subdomain={}&record_id=0&type={}&content={}",
            self.server, self.domain, subdomain, record_type, ip
        );

        log::info!("Updating {} with Yandex", hostname);

        let response = minreq::post(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .with_header("PddToken", &self.token)
            .send()?;

        let status_code = response.status_code;
        let body = response.as_str()?.trim();

        log::debug!("Response status: {}, body: {}", status_code, body);

        if status_code != 200 {
            return Err(format!("HTTP error: {}", status_code).into());
        }

        // Parse JSON response
        if body.contains(r#""success":"ok""#) || body.contains(r#""ok":true"#) {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else if body.contains("error") {
            Err(format!("Yandex API error: {}", body).into())
        } else {
            Err(format!("Unexpected response: {}", body).into())
        }
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.token.is_empty() {
            return Err("password (PDD token) is required for Yandex".into());
        }
        if self.domain.is_empty() {
            return Err("zone_id (domain) is required for Yandex".into());
        }
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "Yandex"
    }
}
