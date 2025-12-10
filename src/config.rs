use serde::Deserialize;
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Config {
    // Protocol/provider selection (ddclient: protocol)
    pub protocol: Option<String>,

    // Authentication (ddclient: login/password)
    pub login: Option<String>,
    pub password: Option<String>,

    // Common settings
    pub server: Option<String>,
    pub zone: Option<String>,
    pub host: Option<String>,
    pub ttl: Option<u32>,

    // Email notifications (emailonly provider)
    pub email: Option<String>,

    // Runtime options
    pub ip: Option<String>,
}

impl Config {
    /// Load configuration from file
    pub fn from_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let settings = config::Config::builder()
            .add_source(config::File::with_name(path))
            .build()?;
        let config: Config = settings.try_deserialize()?;
        Ok(config)
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
            .map(|r| r.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_from_file() {
        let config_content = r#"
        protocol = "cloudflare"
        zone = "example.com"
        login = "token"
        password = "example-api-token"
        host = "www.example.com"
        ttl = 3600
        "#;

        let config_path = "test_config_from_file.ini";
        let mut file = File::create(config_path).expect("Unable to create test config file");
        file.write_all(config_content.as_bytes())
            .expect("Unable to write to test config file");

        let config = Config::from_file(config_path).expect("Failed to read config file");

        assert_eq!(config.protocol.unwrap(), "cloudflare");
        assert_eq!(config.zone.unwrap(), "example.com");
        assert_eq!(config.login.unwrap(), "token");
        assert_eq!(config.password.unwrap(), "example-api-token");
        assert_eq!(config.host.unwrap(), "www.example.com");
        assert_eq!(config.ttl.unwrap(), 3600);

        std::fs::remove_file(config_path).expect("Unable to delete test config file");
    }

    #[test]
    fn test_dns_records_split() {
        let config = Config {
            host: Some("example.com,test.example.com,api.example.com".to_string()),
            ..Default::default()
        };

        let records = config.dns_records();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0], "example.com");
        assert_eq!(records[1], "test.example.com");
        assert_eq!(records[2], "api.example.com");
    }
}
