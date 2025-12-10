use crate::clients::DnsClient;
use crate::config::Config;
use sha1::{Digest, Sha1};
use std::error::Error;
use std::net::IpAddr;
use std::time::{SystemTime, UNIX_EPOCH};

/// NearlyFreeSpeech.NET (NFSN) DNS client
/// Uses NFSN REST API with SHA1 authentication
/// Based on: https://members.nearlyfreespeech.net/wiki/API/Introduction
pub struct NfsnClient {
    server: String,
    login: String,
    api_key: String,
    zone: String,
}

impl NfsnClient {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let login = config.login.as_ref()
            .ok_or("login is required for NFSN")?
            .clone();
        
        let api_key = config.password.as_ref()
            .ok_or("API key (password) is required for NFSN")?
            .clone();
        
        let zone = config.zone.as_ref()
            .ok_or("zone is required for NFSN")?
            .clone();
        
        let server = config.server.clone()
            .unwrap_or_else(|| "https://api.nearlyfreespeech.net".to_string());

        Ok(NfsnClient {
            server,
            login,
            api_key,
            zone,
        })
    }

    /// Generate NFSN authentication header value
    /// Format: login;timestamp;salt;hash
    /// hash = SHA1(login;timestamp;salt;api-key;request-uri;body-hash)
    fn gen_auth_header(&self, path: &str, body: &str) -> String {
        use rand::Rng;
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Generate cryptographically secure 16-character random salt
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let mut rng = rand::thread_rng();
        let salt: String = (0..16)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();
        
        // Hash the body
        let body_hash = hex::encode(Sha1::digest(body.as_bytes()));
        
        // Build hash string: login;timestamp;salt;api-key;request-uri;body-hash
        let hash_string = format!(
            "{};{};{};{};{};{}",
            self.login, timestamp, salt, self.api_key, path, body_hash
        );
        
        let hash = hex::encode(Sha1::digest(hash_string.as_bytes()));
        
        format!("{};{};{};{}", self.login, timestamp, salt, hash)
    }

    /// Make authenticated request to NFSN API
    fn make_request(&self, path: &str, method: &str, body: &str) -> Result<String, Box<dyn Error>> {
        let url = format!("{}{}", self.server, path);
        let auth_header = self.gen_auth_header(path, body);
        
        let request = match method {
            "GET" => minreq::get(&url),
            "POST" => minreq::post(&url)
                .with_header("Content-Type", "application/x-www-form-urlencoded")
                .with_body(body),
            _ => return Err(format!("Unsupported HTTP method: {}", method).into()),
        };
        
        let response = request
            .with_header("User-Agent", crate::USER_AGENT)
            .with_header("X-NFSN-Authentication", auth_header)
            .send()?;
        
        let status = response.status_code;
        let body = response.as_str()?.to_string();
        
        if status >= 200 && status < 300 {
            Ok(body)
        } else {
            // Try to parse error JSON
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                if let Some(error) = json.get("error").and_then(|e| e.as_str()) {
                    return Err(format!("NFSN API error: {}", error).into());
                }
            }
            Err(format!("HTTP {} error: {}", status, body).into())
        }
    }

    /// Extract subdomain name from hostname (strip zone suffix)
    fn extract_name(&self, hostname: &str) -> String {
        if hostname == self.zone {
            String::new()
        } else if let Some(name) = hostname.strip_suffix(&format!(".{}", self.zone)) {
            name.to_string()
        } else {
            hostname.to_string()
        }
    }
}

impl DnsClient for NfsnClient {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        log::info!("Updating {} with NFSN", hostname);

        // Verify hostname is in the zone
        if hostname != self.zone && !hostname.ends_with(&format!(".{}", self.zone)) {
            return Err(format!("{} is outside zone {}", hostname, self.zone).into());
        }

        let name = self.extract_name(hostname);
        
        // Step 1: List existing A records for this name
        let list_path = format!("/dns/{}/listRRs", self.zone);
        let list_body = format!("name={}&type=A", urlencoding::encode(&name));
        let list_resp = self.make_request(&list_path, "POST", &list_body)?;
        
        log::debug!("List response: {}", list_resp);
        
        // Parse JSON response
        let records: Vec<serde_json::Value> = serde_json::from_str(&list_resp)?;
        
        // Step 2: If record exists, remove it first
        if let Some(record) = records.first() {
            if let Some(old_ip) = record.get("data").and_then(|d| d.as_str()) {
                log::info!("Removing old record: {} -> {}", name, old_ip);
                let rm_path = format!("/dns/{}/removeRR", self.zone);
                let rm_body = format!(
                    "name={}&type=A&data={}",
                    urlencoding::encode(&name),
                    urlencoding::encode(old_ip)
                );
                self.make_request(&rm_path, "POST", &rm_body)?;
            }
        }
        
        // Step 3: Add new record
        log::info!("Adding new record: {} -> {}", name, ip);
        let add_path = format!("/dns/{}/addRR", self.zone);
        let add_body = format!(
            "name={}&type=A&data={}&ttl=3600",
            urlencoding::encode(&name),
            ip
        );
        self.make_request(&add_path, "POST", &add_body)?;
        
        log::info!("Successfully updated {} to {}", hostname, ip);
        Ok(())
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.login.is_empty() {
            return Err("login is required for NFSN".into());
        }
        if self.api_key.is_empty() {
            return Err("API key (password) is required for NFSN".into());
        }
        if self.zone.is_empty() {
            return Err("zone is required for NFSN".into());
        }
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "NFSN"
    }
}
