use crate::clients::DnsClient;
use crate::config::Config;
use sha1::{Digest, Sha1};
use std::error::Error;
use std::net::IpAddr;

/// Type alias for Afraid.org record: (hostname, record_type, update_url)
type AfraidRecord = (String, String, String);

/// Afraid.org (FreeDNS) DNS client using v2 API
///
/// Uses two-step process:
/// 1. Get record list with SHA1(login|password)
/// 2. Call record-specific update URL
///
/// API docs: <https://freedns.afraid.org/api/>
pub struct AfraidClient {
    server: String,
    login: String,
    password: String,
}

impl AfraidClient {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let login = config.login.as_ref()
            .ok_or("login is required for Afraid.org")?
            .clone();
        
        let password = config.password.as_ref()
            .ok_or("password is required for Afraid.org")?
            .clone();
        
        let server = config.server.clone()
            .unwrap_or_else(|| "https://freedns.afraid.org".to_string());

        Ok(AfraidClient {
            server,
            login,
            password,
        })
    }

    /// Get list of all records and their update URLs
    /// Returns Vec of (hostname, record_type, update_url)
    fn get_record_list(&self) -> Result<Vec<AfraidRecord>, Box<dyn Error>> {
        // Step 1: Generate SHA1 hash of "login|password"
        let credentials = format!("{}|{}", self.login, self.password);
        let hash = hex::encode(Sha1::digest(credentials.as_bytes()));
        
        // Step 2: Request record list
        let url = format!("{}/api/?action=getdyndns&v=2&sha={}", self.server, hash);
        
        log::debug!("Fetching record list from Afraid.org");
        let response = minreq::get(&url)
            .with_header("User-Agent", crate::USER_AGENT)
            .send()?;
        
        if response.status_code != 200 {
            return Err(format!("HTTP {} error fetching record list", response.status_code).into());
        }
        
        let body = response.as_str()?;
        log::debug!("Record list response: {} lines", body.lines().count());
        
        // Step 3: Parse pipe-delimited response
        // Format: hostname|current_ip|update_url
        let mut records = Vec::new();
        for line in body.lines() {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 3 {
                records.push((
                    parts[0].to_string(),
                    parts[1].to_string(),
                    parts[2].to_string(),
                ));
                log::debug!("Found record: {} -> {} (update URL present)", parts[0], parts[1]);
            }
        }
        
        if records.is_empty() {
            return Err("No records found in Afraid.org account".into());
        }
        
        Ok(records)
    }
}

impl DnsClient for AfraidClient {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        log::info!("Updating {} with Afraid.org", hostname);

        // Get all records
        let records = self.get_record_list()?;
        
        // Find matching record for this hostname and IP type
        let is_ipv6 = ip.is_ipv6();
        let matching_record = records.iter()
            .find(|(host, current_ip, _)| {
                if host != hostname {
                    return false;
                }
                // Match IP type: NULL can be updated with IPv4, otherwise must match
                if current_ip == "NULL" && !is_ipv6 {
                    return true;
                }
                // Check if current IP matches our IP type
                current_ip.contains(':') == is_ipv6
            });
        
        let (_, current_ip, update_url) = matching_record
            .ok_or_else(|| format!("No matching {} record found for {}", 
                                  if is_ipv6 { "AAAA" } else { "A" }, 
                                  hostname))?;
        
        // Check if update is needed
        if current_ip == &ip.to_string() {
            log::info!("Record {} already set to {}, no update needed", hostname, ip);
            return Ok(());
        }
        
        // Call update URL with new address
        let update_url = format!("{}&address={}", update_url, ip);
        log::debug!("Calling update URL (credentials redacted)");
        
        let response = minreq::get(&update_url)
            .with_header("User-Agent", crate::USER_AGENT)
            .send()?;
        
        let status_code = response.status_code;
        let body = response.as_str()?.trim();
        
        log::debug!("Update response status: {}, body: {}", status_code, body);
        
        if status_code != 200 {
            return Err(format!("HTTP {} error during update", status_code).into());
        }
        
        // Check response for success
        if body.contains("Updated") || body.contains("has not changed") {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else if body.contains("fail") || body.contains("ERROR") {
            Err(format!("Afraid.org error: {}", body).into())
        } else {
            Err(format!("Unexpected response: {}", body).into())
        }
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.login.is_empty() {
            return Err("login is required for Afraid.org".into());
        }
        if self.password.is_empty() {
            return Err("password is required for Afraid.org".into());
        }
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "Afraid.org"
    }
}
