/// ddclient-compatible configuration parser
/// 
/// This module parses the ddclient configuration format:
/// - key=value pairs (comma-separated or on separate lines)
/// - Backslash line continuation
/// - Global defaults that apply to subsequent blocks
/// - Host blocks terminated by bare hostnames
///
/// Example ddclient format:
/// ```
/// protocol=cloudflare, \
/// zone=example.com, \
/// login=token, \
/// password=key \
/// host1.example.com,host2.example.com
/// ```
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;

// Main Config struct used throughout the codebase
#[derive(Debug, Clone, Default)]
pub struct Config {
    pub protocol: Option<String>,
    pub login: Option<String>,
    pub password: Option<String>,
    pub server: Option<String>,
    pub zone: Option<String>,
    pub host: Option<String>,
    pub ttl: Option<u32>,
    pub email: Option<String>,
    pub ip: Option<String>,
}

impl Config {
    /// Load configuration from file (ddclient format)
    pub fn from_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let ddclient_config = DdclientConfig::from_file(path)?;
        
        // For now, take the first config block
        // TODO: Support multiple host blocks
        if let Some(host_config) = ddclient_config.configs.first() {
            Ok(Config::from(host_config.clone()))
        } else {
            Err("No valid configuration found in file".into())
        }
    }

    /// Merge configuration from file with CLI arguments
    /// CLI arguments take precedence over file configuration
    pub fn merge(file_config: Option<Self>, args: &crate::args::Args) -> Self {
        let base = file_config.unwrap_or_default();

        Config {
            protocol: Some(args.protocol.clone()),
            login: args.login.clone().or(base.login),
            password: args.password.clone().or(base.password),
            server: args.server.clone().or(base.server),
            zone: args.zone.clone().or(base.zone),
            host: args.host.clone().or(base.host),
            ttl: args.ttl.or(base.ttl),
            email: base.email,
            ip: args.ip.clone().or(base.ip),
        }
    }

    /// Load and merge configuration
    pub fn load(args: &crate::args::Args) -> Result<Self, Box<dyn Error>> {
        let default_config_path = "rddclient.conf";
        let config_file = args.file.as_deref().unwrap_or(default_config_path);

        let file_config = if Path::new(config_file).exists() {
            Some(Self::from_file(config_file)?)
        } else {
            None
        };

        Ok(Self::merge(file_config, args))
    }

    /// Validate that required fields are present
    pub fn validate(&self) -> Result<(), Box<dyn Error>> {
        if self.host.is_none() || self.host.as_ref().unwrap().is_empty() {
            return Err("Host is required (use --host)".into());
        }

        Ok(())
    }

    /// Get DNS records as a vector
    pub fn dns_records(&self) -> Vec<String> {
        self.host
            .as_ref()
            .map(|h| {
                h.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl From<HostConfig> for Config {
    fn from(hc: HostConfig) -> Self {
        Config {
            protocol: hc.protocol,
            login: hc.login,
            password: hc.password,
            server: hc.server,
            zone: hc.zone,
            host: hc.host,
            ttl: hc.ttl,
            email: hc.email,
            ip: None,
        }
    }
}

// Internal parsing structures
#[derive(Debug, Clone, Default)]
struct DdclientConfig {
    configs: Vec<HostConfig>,
}

#[derive(Debug, Clone, Default)]
struct HostConfig {
    protocol: Option<String>,
    login: Option<String>,
    password: Option<String>,
    server: Option<String>,
    zone: Option<String>,
    host: Option<String>,
    ttl: Option<u32>,
    email: Option<String>,
    
    // ddclient-specific fields (for future compatibility)
    #[allow(dead_code)]
    use_method: Option<String>,  // web, if, cmd, fw
    #[allow(dead_code)]
    web: Option<String>,
    #[allow(dead_code)]
    ssl: Option<bool>,
}

impl DdclientConfig {
    /// Parse ddclient configuration file
    pub fn from_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let content = fs::read_to_string(path)?;
        Self::parse(&content)
    }

    /// Parse ddclient configuration from string
    pub fn parse(content: &str) -> Result<Self, Box<dyn Error>> {
        let mut configs = Vec::new();
        let mut global_defaults: HashMap<String, String> = HashMap::new();
        let mut current_block: HashMap<String, String> = HashMap::new();
        
        // Join lines that end with backslash
        let normalized = Self::join_continued_lines(content);
        
        for line in normalized.lines() {
            let line = line.trim();
            
            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            // Check if this line contains bare hostnames (no = sign)
            if !line.contains('=') {
                // This is a hostname list - create configs for each host
                let hosts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
                
                for host in hosts {
                    if !host.is_empty() {
                        // Merge global defaults with current block
                        let mut config_map = global_defaults.clone();
                        config_map.extend(current_block.clone());
                        config_map.insert("host".to_string(), host.to_string());
                        
                        configs.push(Self::map_to_config(config_map));
                    }
                }
                
                // Reset current block after processing hosts
                current_block.clear();
                continue;
            }
            
            // Parse key=value pairs (comma-separated)
            // But also handle trailing hostnames after the last comma
            let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
            let mut found_hostnames = Vec::new();
            
            for part in parts {
                if let Some((key, value)) = part.split_once('=') {
                    let key = key.trim();
                    let value = value.trim();
                    
                    // Check if value contains a space followed by something that's not a comment
                    if let Some(space_pos) = value.find(' ') {
                        let (actual_value, rest) = value.split_at(space_pos);
                        let rest = rest.trim();
                        
                        // If what follows is not a comment, treat it as a hostname
                        if !rest.starts_with('#') && !rest.is_empty() {
                            if configs.is_empty() && current_block.is_empty() {
                                global_defaults.insert(key.to_string(), actual_value.trim().to_string());
                            } else {
                                current_block.insert(key.to_string(), actual_value.trim().to_string());
                            }
                            found_hostnames.push(rest);
                        } else {
                            // It's a comment or empty, just use the actual value
                            if configs.is_empty() && current_block.is_empty() {
                                global_defaults.insert(key.to_string(), actual_value.trim().to_string());
                            } else {
                                current_block.insert(key.to_string(), actual_value.trim().to_string());
                            }
                        }
                    } else {
                        // Normal key=value
                        if configs.is_empty() && current_block.is_empty() {
                            global_defaults.insert(key.to_string(), value.to_string());
                        } else {
                            current_block.insert(key.to_string(), value.to_string());
                        }
                    }
                } else if !part.is_empty() && !part.starts_with('#') {
                    // This is a bare hostname (not a comment)
                    found_hostnames.push(part);
                }
            }
            
            // Process any hostnames found
            let has_hostnames = !found_hostnames.is_empty();
            for host in found_hostnames {
                if !host.is_empty() {
                    let mut config_map = global_defaults.clone();
                    config_map.extend(current_block.clone());
                    config_map.insert("host".to_string(), host.to_string());
                    configs.push(Self::map_to_config(config_map));
                }
            }
            
            // Reset current block if we processed hostnames
            if has_hostnames {
                current_block.clear();
            }
        }
        
        // If we have a current block without hostnames, use it as a single config
        if !current_block.is_empty() {
            let mut config_map = global_defaults.clone();
            config_map.extend(current_block);
            configs.push(Self::map_to_config(config_map));
        }
        
        Ok(DdclientConfig { configs })
    }
    
    /// Join lines that end with backslash
    fn join_continued_lines(content: &str) -> String {
        let mut result = String::new();
        let mut current_line = String::new();
        
        for line in content.lines() {
            let trimmed = line.trim_end();
            
            if let Some(stripped) = trimmed.strip_suffix('\\') {
                // Remove backslash and append
                current_line.push_str(stripped);
                current_line.push(' ');
            } else {
                // Complete the line
                current_line.push_str(trimmed);
                result.push_str(&current_line);
                result.push('\n');
                current_line.clear();
            }
        }
        
        // Add any remaining line
        if !current_line.is_empty() {
            result.push_str(&current_line);
            result.push('\n');
        }
        
        result
    }
    
    /// Convert HashMap to HostConfig
    fn map_to_config(map: HashMap<String, String>) -> HostConfig {
        HostConfig {
            protocol: map.get("protocol").cloned(),
            login: map.get("login").cloned(),
            password: map.get("password").cloned(),
            server: map.get("server").cloned(),
            zone: map.get("zone").cloned(),
            host: map.get("host").cloned(),
            ttl: map.get("ttl").and_then(|s| s.parse().ok()),
            email: map.get("email").cloned(),
            use_method: map.get("use").cloned(),
            web: map.get("web").cloned(),
            ssl: map.get("ssl").and_then(|s| match s.to_lowercase().as_str() {
                "yes" | "true" | "1" => Some(true),
                "no" | "false" | "0" => Some(false),
                _ => None,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_config() {
        let config = r#"
protocol=cloudflare
zone=example.com
login=token
password=secret
host1.example.com
"#;
        
        let parsed = DdclientConfig::parse(config).unwrap();
        assert_eq!(parsed.configs.len(), 1);
        assert_eq!(parsed.configs[0].protocol.as_deref(), Some("cloudflare"));
        assert_eq!(parsed.configs[0].zone.as_deref(), Some("example.com"));
        assert_eq!(parsed.configs[0].host.as_deref(), Some("host1.example.com"));
    }

    #[test]
    fn test_parse_comma_separated() {
        let config = r#"
protocol=cloudflare, zone=example.com, login=token, password=secret
host1.example.com,host2.example.com
"#;
        
        let parsed = DdclientConfig::parse(config).unwrap();
        assert_eq!(parsed.configs.len(), 2);
        assert_eq!(parsed.configs[0].host.as_deref(), Some("host1.example.com"));
        assert_eq!(parsed.configs[1].host.as_deref(), Some("host2.example.com"));
    }

    #[test]
    fn test_parse_backslash_continuation() {
        let config = r#"
protocol=cloudflare, \
zone=example.com, \
login=token, \
password=secret \
host.example.com
"#;
        
        let parsed = DdclientConfig::parse(config).unwrap();
        assert_eq!(parsed.configs.len(), 1);
        assert_eq!(parsed.configs[0].protocol.as_deref(), Some("cloudflare"));
        assert_eq!(parsed.configs[0].host.as_deref(), Some("host.example.com"));
    }

    #[test]
    fn test_global_defaults() {
        let config = r#"
protocol=dyndns2
login=user
password=pass

server=server1.com
host1.example.com

server=server2.com
host2.example.com
"#;
        
        let parsed = DdclientConfig::parse(config).unwrap();
        assert_eq!(parsed.configs.len(), 2);
        
        // Both should have global defaults
        assert_eq!(parsed.configs[0].protocol.as_deref(), Some("dyndns2"));
        assert_eq!(parsed.configs[1].protocol.as_deref(), Some("dyndns2"));
        
        // But different servers
        assert_eq!(parsed.configs[0].server.as_deref(), Some("server1.com"));
        assert_eq!(parsed.configs[1].server.as_deref(), Some("server2.com"));
    }

    #[test]
    fn test_ignore_comments() {
        let config = r#"
# This is a comment
protocol=cloudflare  # inline comment is NOT supported
zone=example.com
# Another comment
host.example.com
"#;
        
        let parsed = DdclientConfig::parse(config).unwrap();
        assert_eq!(parsed.configs.len(), 1);
    }

    #[test]
    fn test_ttl_parsing() {
        let config = r#"
protocol=cloudflare
zone=example.com
ttl=300
host.example.com
"#;
        
        let parsed = DdclientConfig::parse(config).unwrap();
        assert_eq!(parsed.configs[0].ttl, Some(300));
    }

    #[test]
    fn test_ssl_parsing() {
        let config = r#"
protocol=dyndns2
ssl=yes
host1.example.com

ssl=no
host2.example.com
"#;
        
        let parsed = DdclientConfig::parse(config).unwrap();
        assert_eq!(parsed.configs[0].ssl, Some(true));
        assert_eq!(parsed.configs[1].ssl, Some(false));
    }

    #[test]
    fn test_parse_interval() {
        assert_eq!(parse_interval("30s").unwrap(), 30);
        assert_eq!(parse_interval("5m").unwrap(), 300);
        assert_eq!(parse_interval("2h").unwrap(), 7200);
        assert_eq!(parse_interval("1d").unwrap(), 86400);
        assert_eq!(parse_interval("25d").unwrap(), 2160000);
        assert!(parse_interval("invalid").is_err());
        assert!(parse_interval("").is_err());
    }
}

/// Parse interval string (e.g., "30s", "5m", "1h", "25d") to seconds
/// ddclient-compatible format
pub fn parse_interval(interval: &str) -> Result<u64, Box<dyn Error>> {
    if interval.is_empty() {
        return Err("Interval cannot be empty".into());
    }

    let interval = interval.trim();
    let len = interval.len();
    
    if len < 2 {
        return Err(format!("Invalid interval format: '{}'", interval).into());
    }

    let (num_str, unit) = interval.split_at(len - 1);
    let num: u64 = num_str.parse()
        .map_err(|_| format!("Invalid number in interval: '{}'", num_str))?;

    let seconds = match unit {
        "s" => num,
        "m" => num * 60,
        "h" => num * 3600,
        "d" => num * 86400,
        _ => return Err(format!("Invalid interval unit '{}'. Use s, m, h, or d", unit).into()),
    };

    Ok(seconds)
}
