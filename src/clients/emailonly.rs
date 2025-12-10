use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;
use std::process::{Command, Stdio};
use std::io::Write;

/// Email-only notification client
/// Does NOT update any DNS records, only sends email notifications when IP changes
/// Requires system sendmail to be configured
pub struct EmailonlyClient {
    email: String,
    hostname: String,
}

impl EmailonlyClient {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let email = config.email.as_ref()
            .ok_or("email address is required for emailonly provider")?
            .clone();
        
        // Get hostname for email subject
        let hostname = std::env::var("HOSTNAME")
            .or_else(|_| std::env::var("HOST"))
            .unwrap_or_else(|_| "localhost".to_string());

        Ok(EmailonlyClient {
            email,
            hostname,
        })
    }

    fn send_email(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        // Construct email body
        let body = format!(
            "Host IP addresses:\n{:>30}  {}\n\n-- \n   {}@{} (version {})",
            hostname,
            ip,
            env!("CARGO_PKG_NAME"),
            self.hostname,
            env!("CARGO_PKG_VERSION")
        );

        // Construct email headers and body
        let email_content = format!(
            "To: {}\nSubject: status report from {}@{}\n\r\n{}\n",
            self.email,
            env!("CARGO_PKG_NAME"),
            self.hostname,
            body
        );

        // Spawn sendmail process
        let mut child = Command::new("sendmail")
            .arg("-oi")
            .arg(&self.email)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn sendmail: {}. Ensure sendmail is installed and in PATH.", e))?;

        // Write email content to stdin
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(email_content.as_bytes())
                .map_err(|e| format!("Failed to write to sendmail stdin: {}", e))?;
        }

        // Wait for sendmail to complete
        let status = child.wait()
            .map_err(|e| format!("Failed to wait for sendmail: {}", e))?;

        if !status.success() {
            let stderr = if let Some(mut stderr) = child.stderr.take() {
                let mut buf = String::new();
                use std::io::Read;
                stderr.read_to_string(&mut buf).ok();
                buf
            } else {
                String::new()
            };
            return Err(format!("sendmail failed with status: {}. Error: {}", status, stderr).into());
        }

        log::info!("Email notification sent to {} for host {}", self.email, hostname);
        Ok(())
    }
}

impl DnsClient for EmailonlyClient {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        log::info!("Email-only mode: sending notification for {} -> {}", hostname, ip);
        
        // Send email notification
        self.send_email(hostname, ip)?;
        
        // Always return success - we don't actually update DNS
        Ok(())
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.email.is_empty() {
            return Err("email address is required for emailonly provider".into());
        }
        
        // Check if sendmail is available
        match Command::new("sendmail").arg("-h").output() {
            Ok(_) => Ok(()),
            Err(_) => Err("sendmail command not found. Please install sendmail, postfix, or another MTA.".into()),
        }
    }

    fn provider_name(&self) -> &str {
        "EmailOnly"
    }
}
