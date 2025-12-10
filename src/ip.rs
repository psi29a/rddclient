use std::error::Error;
use std::net::IpAddr;

/// Get external IP address from a public service
pub fn get_external_ip() -> Result<IpAddr, Box<dyn Error>> {
    // Try multiple services in case one is down
    let services = [
        "http://checkip.amazonaws.com",
        "http://icanhazip.com",
        "http://ifconfig.me/ip",
    ];

    let mut last_error = None;

    for service in &services {
        match try_service(service) {
            Ok(ip) => return Ok(ip),
            Err(e) => {
                log::debug!("Failed to get IP from {}: {}", service, e);
                last_error = Some(e);
            }
        }
    }

    Err(last_error.unwrap_or_else(|| "Failed to get external IP from any service".into()))
}

/// Try to get IP from a specific service
fn try_service(url: &str) -> Result<IpAddr, Box<dyn Error>> {
    let resp = minreq::get(url)
        .with_timeout(10)
        .send()?;
    let ip: IpAddr = resp.as_str()?.trim().parse()?;
    Ok(ip)
}

/// Parse and validate a provided IP address string
pub fn parse_ip(ip_str: &str) -> Result<IpAddr, Box<dyn Error>> {
    ip_str.parse().map_err(|e| {
        format!("'{}' is an invalid IP address: {}", ip_str, e).into()
    })
}

/// Get IP address - either from provided string or auto-detect
pub fn get_ip(provided: Option<&str>) -> Result<IpAddr, Box<dyn Error>> {
    match provided {
        Some(ip_str) => parse_ip(ip_str),
        None => get_external_ip(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_ipv4() {
        let result = parse_ip("8.8.8.8");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), "8.8.8.8");
    }

    #[test]
    fn test_parse_valid_ipv6() {
        let result = parse_ip("2001:4860:4860::8888");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_invalid_ip() {
        let result = parse_ip("invalid_ip");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_empty_string() {
        let result = parse_ip("");
        assert!(result.is_err());
    }

    #[test]
    fn test_ipv4_edge_cases() {
        // Test boundary values
        assert!(parse_ip("0.0.0.0").is_ok());
        assert!(parse_ip("255.255.255.255").is_ok());
        assert_eq!(parse_ip("0.0.0.0").unwrap().to_string(), "0.0.0.0");
        assert_eq!(parse_ip("255.255.255.255").unwrap().to_string(), "255.255.255.255");
    }

    #[test]
    fn test_ipv4_out_of_range() {
        // Test values outside valid range
        assert!(parse_ip("256.1.1.1").is_err());
        assert!(parse_ip("1.256.1.1").is_err());
        assert!(parse_ip("1.1.256.1").is_err());
        assert!(parse_ip("1.1.1.256").is_err());
        assert!(parse_ip("999.999.999.999").is_err());
    }

    #[test]
    fn test_ipv4_malformed() {
        // Test various malformed addresses
        assert!(parse_ip("192.168.1").is_err());
        assert!(parse_ip("192.168.1.1.1").is_err());
        assert!(parse_ip("192.168..1").is_err());
        assert!(parse_ip(".192.168.1.1").is_err());
        assert!(parse_ip("192.168.1.1.").is_err());
        assert!(parse_ip("192.168.1.abc").is_err());
    }

    #[test]
    fn test_ipv6_compressed() {
        // Test compressed notation
        assert!(parse_ip("::1").is_ok());
        assert_eq!(parse_ip("::1").unwrap().to_string(), "::1");
        
        assert!(parse_ip("::").is_ok());
        assert_eq!(parse_ip("::").unwrap().to_string(), "::");
        
        assert!(parse_ip("2001:db8::1").is_ok());
        assert!(parse_ip("fe80::1").is_ok());
    }

    #[test]
    fn test_ipv6_full() {
        // Test full addresses
        let full_ipv6 = "2001:0db8:0000:0000:0000:0000:0000:0001";
        assert!(parse_ip(full_ipv6).is_ok());
        
        // Rust normalizes to compressed form
        let parsed = parse_ip(full_ipv6).unwrap().to_string();
        assert!(parsed.contains("2001") && parsed.contains("db8"));
    }

    #[test]
    fn test_ipv6_mixed() {
        // Test IPv6 with embedded IPv4
        assert!(parse_ip("::ffff:192.0.2.1").is_ok());
        assert!(parse_ip("64:ff9b::192.0.2.1").is_ok());
    }

    #[test]
    fn test_ipv4_whitespace() {
        // Test trimming of whitespace
        assert!(parse_ip(" 8.8.8.8 ").is_err(), "Should reject whitespace (requires explicit trim)");
        
        // If IP is properly trimmed before parsing, it should work
        let trimmed = " 8.8.8.8 ".trim();
        assert!(parse_ip(trimmed).is_ok());
        assert_eq!(parse_ip(trimmed).unwrap().to_string(), "8.8.8.8");
    }

    #[test]
    fn test_get_ip_with_provided() {
        // Test get_ip with provided IP
        let result = get_ip(Some("1.2.3.4"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), "1.2.3.4");
    }

    #[test]
    fn test_get_ip_with_invalid_provided() {
        // Test get_ip with invalid provided IP
        let result = get_ip(Some("invalid"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid IP address"));
    }
}
