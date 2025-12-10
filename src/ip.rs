use std::error::Error;
use std::net::IpAddr;
use std::process::Command;

/// IP detection method
#[derive(Debug, Clone, PartialEq)]
pub enum IpDetectionMethod {
    /// Manual IP address
    Manual(String),
    /// Web service (default)
    Web(Option<String>),
    /// Network interface
    Interface(String),
    /// Execute command
    Command(String),
}

impl Default for IpDetectionMethod {
    /// Default IP detection method: web-based lookup using the built-in services.
    ///
    /// # Examples
    ///
    /// ```
    /// let d = IpDetectionMethod::default();
    /// assert_eq!(d, IpDetectionMethod::Web(None));
    /// ```
    fn default() -> Self {
        Self::Web(None)
    }
}

/// Obtain the machine's external IP by querying public web services.
///
/// Queries multiple well-known public IP services in sequence and returns the first successfully parsed IP address.
/// On success returns the discovered external `IpAddr`; on failure returns an error describing the last failure.
///
/// # Examples
///
/// ```
/// let ip = get_external_ip().expect("failed to get external IP");
/// println!("{}", ip);
/// ```
pub fn get_external_ip() -> Result<IpAddr, Box<dyn Error>> {
    // Try multiple services in case one is down
    // Matches ddclient's built-in web services for compatibility
    let services = [
        "https://api.ipify.org",           // ipify (most popular, supports both v4/v6)
        "https://checkip.dns.he.net",      // Hurricane Electric
        "http://checkip.amazonaws.com",    // AWS (reliable)
        "http://icanhazip.com",            // Simple service
        "https://ip4only.me/api",          // IPv4-specific
        "https://ipv4.nsupdate.info/myip", // nsupdate.info
        "http://ifconfig.me/ip",           // ifconfig.me
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

/// Fetches an IP address from a single HTTP service URL.
///
/// The provided `url` is expected to return a plain IP address in the response body.
/// Returns a parsed `IpAddr` on success or an error when the request fails or the body does not contain a valid IP.
///
/// # Examples
///
/// ```
/// let ip = try_service("https://api.ipify.org").unwrap();
/// assert!(matches!(ip, std::net::IpAddr::V4(_) | std::net::IpAddr::V6(_)));
/// ```
fn try_service(url: &str) -> Result<IpAddr, Box<dyn Error>> {
    let resp = minreq::get(url)
        .with_timeout(10)
        .send()?;
    let ip: IpAddr = resp.as_str()?.trim().parse()?;
    Ok(ip)
}

/// Parses an IP address from the provided string and returns it.
///
/// # Returns
/// `IpAddr` if `ip_str` is a valid IPv4 or IPv6 address, an error with a descriptive message otherwise.
///
/// # Examples
///
/// ```
/// let ip = parse_ip("127.0.0.1").unwrap();
/// assert_eq!(ip.to_string(), "127.0.0.1");
/// ```
pub fn parse_ip(ip_str: &str) -> Result<IpAddr, Box<dyn Error>> {
    ip_str.parse().map_err(|e| {
        format!("'{}' is an invalid IP address: {}", ip_str, e).into()
    })
}

/// Obtain the first non-loopback IP address assigned to a network interface.
///
/// This queries platform-specific system utilities to find an IPv4 or IPv6 address for `interface`.
/// On success returns the first valid, non-loopback address found; on failure returns an error.
///
/// # Parameters
///
/// - `interface`: name of the network interface (for example `"eth0"` or `"en0"`).
///
/// # Returns
///
/// `Ok(IpAddr)` with the interface's address, `Err` if no valid address is found or the underlying system commands fail.
///
/// # Examples
///
/// ```no_run
/// use std::net::IpAddr;
/// # // function under test is assumed to be in the same crate
/// let res: Result<IpAddr, Box<dyn std::error::Error>> = get_ip_from_interface("eth0");
/// match res {
///     Ok(ip) => println!("interface ip: {}", ip),
///     Err(e) => eprintln!("could not determine IP: {}", e),
/// }
/// ```
pub fn get_ip_from_interface(interface: &str) -> Result<IpAddr, Box<dyn Error>> {
    #[cfg(target_os = "linux")]
    {
        // Try `ip` command first (modern Linux)
        if let Ok(output) = Command::new("ip")
            .args(["-o", "addr", "show", "dev", interface, "scope", "global"])
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Some(ip) = extract_ip_from_output(&stdout) {
                    return Ok(ip);
                }
            }
        }

        // Fall back to `ifconfig`
        if let Ok(output) = Command::new("ifconfig").arg(interface).output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Some(ip) = extract_ip_from_output(&stdout) {
                    return Ok(ip);
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        // Try `ifconfig` on macOS
        if let Ok(output) = Command::new("ifconfig").arg(interface).output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Some(ip) = extract_ip_from_output(&stdout) {
                    return Ok(ip);
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Use `ipconfig` on Windows
        if let Ok(output) = Command::new("ipconfig").output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                // Windows ipconfig output is more complex, look for the interface
                if let Some(ip) = extract_ip_from_windows_output(&stdout, interface) {
                    return Ok(ip);
                }
            }
        }
    }

    Err(format!("Failed to get IP from interface '{}'", interface).into())
}

/// Extracts the first non-loopback, non-multicast IP address found in
/// Unix-style command output (e.g., `ip addr` or `ifconfig`).
///
/// Scans each line for the tokens `inet` or `inet6`, takes the following
/// whitespace-separated token (stripping any CIDR suffix like `/24`),
/// validates it as an `IpAddr`, and returns the first address that is
/// neither loopback nor multicast. Returns `None` if no suitable address
/// is found.
///
/// # Examples
///
/// ```
/// use std::net::IpAddr;
/// let output = r#"
/// 2: eth0: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500 qdisc mq state UP group default qlen 1000
///     inet 192.0.2.42/24 brd 192.0.2.255 scope global eth0
///     inet6 fe80::1e2f:65ff:fe9b:7c3a/64 scope link
/// "#;
/// assert_eq!(
///     super::extract_ip_from_output(output),
///     Some("192.0.2.42".parse::<IpAddr>().unwrap())
/// );
/// ```
fn extract_ip_from_output(output: &str) -> Option<IpAddr> {
    use std::str::FromStr;

    for line in output.lines() {
        // Look for "inet " (IPv4) or "inet6 " (IPv6)
        if let Some(inet_pos) = line.find("inet ").or_else(|| line.find("inet6 ")) {
            let after_inet = &line[inet_pos..];
            // Extract the IP address (next whitespace-separated token)
            for word in after_inet.split_whitespace().skip(1) {
                // Remove CIDR notation if present (/24, /64, etc.)
                let ip_str = word.split('/').next().unwrap_or(word);
                if let Ok(ip) = IpAddr::from_str(ip_str) {
                    // Skip loopback and link-local addresses
                    if !ip.is_loopback() && !ip.is_multicast() {
                        return Some(ip);
                    }
                }
            }
        }
    }
    None
}

/// Extracts the first non-loopback IPv4 or IPv6 address assigned to a named interface from Windows `ipconfig` output.
///
/// The function scans `ipconfig` output for the section matching `interface_name` and returns the first valid IP address
/// found under "IPv4 Address" or "IPv6 Address". Returns `None` if no suitable address is located.
///
/// # Examples
///
/// ```
/// use std::net::IpAddr;
///
/// let sample = r#"
/// Ethernet adapter Ethernet:
///
///    Connection-specific DNS Suffix  . : example.local
///    IPv4 Address. . . . . . . . . . . : 192.0.2.10
///    Subnet Mask . . . . . . . . . . . : 255.255.255.0
///    Default Gateway . . . . . . . . . : 192.0.2.1
/// "#;
///
/// // The real function is compiled only on Windows; this demonstrates expected behavior.
/// if let Some(ip) = extract_ip_from_windows_output(sample, "Ethernet") {
///     assert_eq!(ip, IpAddr::from([192, 0, 2, 10]));
/// }
/// ```
#[cfg(target_os = "windows")]
fn extract_ip_from_windows_output(output: &str, interface_name: &str) -> Option<IpAddr> {
    use std::str::FromStr;

    let mut in_target_interface = false;

    for line in output.lines() {
        let trimmed = line.trim();

        // Check if we're entering the target interface section
        if trimmed.contains(interface_name) {
            in_target_interface = true;
            continue;
        }

        // Check if we're leaving an interface section (empty line or new adapter)
        if in_target_interface && (trimmed.is_empty() || trimmed.starts_with("Ethernet adapter") || trimmed.starts_with("Wireless LAN adapter")) {
            if !trimmed.contains(interface_name) {
                in_target_interface = false;
            }
        }

        // Extract IP if we're in the target interface
        if in_target_interface && (trimmed.starts_with("IPv4 Address") || trimmed.starts_with("IPv6 Address")) {
            if let Some(colon_pos) = trimmed.find(':') {
                let ip_part = trimmed[colon_pos + 1..].trim();
                // Remove any trailing info like "(Preferred)"
                let ip_str = ip_part.split_whitespace().next().unwrap_or(ip_part);
                if let Ok(ip) = IpAddr::from_str(ip_str) {
                    if !ip.is_loopback() {
                        return Some(ip);
                    }
                }
            }
        }
    }
    None
}

/// Obtain an IP address by running an external command and parsing its stdout.
///
/// The `cmd` string is split on ASCII whitespace; the first token is treated as the program
/// and the remaining tokens as its arguments. The command must exit successfully and its
/// trimmed stdout must contain a valid IPv4 or IPv6 address.
///
/// # Examples
///
/// ```
/// let ip = get_ip_from_command("echo 1.2.3.4").unwrap();
/// assert_eq!(ip.to_string(), "1.2.3.4");
/// ```
pub fn get_ip_from_command(cmd: &str) -> Result<IpAddr, Box<dyn Error>> {
    // Parse command into program and args
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return Err("Empty command".into());
    }

    let program = parts[0];
    let args = &parts[1..];

    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to execute command '{}': {}", cmd, e))?;

    if !output.status.success() {
        return Err(format!("Command '{}' failed with status: {}", cmd, output.status).into());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let trimmed = stdout.trim();

    // Try to parse the output as an IP address
    parse_ip(trimmed)
}

/// Selects an IP detection strategy and returns the resolved IP address.
///
/// Returns the resolved `IpAddr` when detection succeeds, or an error describing the failure.
///
/// # Examples
///
/// ```
/// use std::net::IpAddr;
/// let method = crate::IpDetectionMethod::Manual("1.2.3.4".to_string());
/// let ip = crate::get_ip_with_method(&method).unwrap();
/// assert_eq!(ip, "1.2.3.4".parse::<IpAddr>().unwrap());
/// ```
pub fn get_ip_with_method(method: &IpDetectionMethod) -> Result<IpAddr, Box<dyn Error>> {
    match method {
        IpDetectionMethod::Manual(ip_str) => parse_ip(ip_str),
        IpDetectionMethod::Web(Some(url)) => {
            // Use custom web service
            try_service(url)
        }
        IpDetectionMethod::Web(None) => {
            // Use default web services
            get_external_ip()
        }
        IpDetectionMethod::Interface(iface) => get_ip_from_interface(iface),
        IpDetectionMethod::Command(cmd) => get_ip_from_command(cmd),
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
        // Test get_ip_with_method with Manual detection
        let method = IpDetectionMethod::Manual("1.2.3.4".to_string());
        let result = get_ip_with_method(&method);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), "1.2.3.4");
    }

    #[test]
    fn test_get_ip_with_invalid_provided() {
        // Test get_ip_with_method with invalid IP
        let method = IpDetectionMethod::Manual("invalid".to_string());
        let result = get_ip_with_method(&method);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid IP address"));
    }

    #[test]
    fn test_ip_detection_method_manual() {
        let method = IpDetectionMethod::Manual("8.8.8.8".to_string());
        let result = get_ip_with_method(&method);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), "8.8.8.8");
    }

    #[test]
    fn test_ip_detection_method_web_default() {
        let method = IpDetectionMethod::Web(None);
        // This test requires internet connectivity
        // Just verify it doesn't panic
        let _ = get_ip_with_method(&method);
    }

    #[test]
    fn test_get_ip_from_command() {
        // Test with echo command
        let result = get_ip_from_command("echo 1.2.3.4");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), "1.2.3.4");
    }

    #[test]
    fn test_get_ip_from_command_invalid() {
        let result = get_ip_from_command("echo invalid_ip");
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_ip_from_output_ipv4() {
        let output = "2: eth0: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500\n    inet 192.168.1.100/24 brd 192.168.1.255 scope global eth0";
        let ip = extract_ip_from_output(output);
        assert!(ip.is_some());
        assert_eq!(ip.unwrap().to_string(), "192.168.1.100");
    }

    #[test]
    fn test_extract_ip_from_output_ipv6() {
        let output = "2: eth0: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500\n    inet6 2001:db8::1/64 scope global";
        let ip = extract_ip_from_output(output);
        assert!(ip.is_some());
        assert!(ip.unwrap().is_ipv6());
    }

    #[test]
    fn test_extract_ip_from_ifconfig() {
        let output = "eth0: flags=4163<UP,BROADCAST,RUNNING,MULTICAST>  mtu 1500\n        inet 10.0.0.5  netmask 255.255.255.0  broadcast 10.0.0.255";
        let ip = extract_ip_from_output(output);
        assert!(ip.is_some());
        assert_eq!(ip.unwrap().to_string(), "10.0.0.5");
    }
}