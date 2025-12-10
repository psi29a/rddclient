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
}
