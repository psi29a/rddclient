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
    /// Create an EmailonlyClient from a Config.
    ///
    /// The returned client uses `config.email` as the recipient address and determines
    /// a hostname for email subject/body by reading the `HOSTNAME` environment variable,
    /// then `HOST`, falling back to `"localhost"` if neither is set.
    ///
    /// Returns an error if `config.email` is not present.
    ///
    /// # Examples
    ///
    /// ```
    /// // assuming a Config type with a public `email: Option<String>` field
    /// let cfg = Config { email: Some("ops@example.com".into()), ..Default::default() };
    /// let client = EmailonlyClient::new(&cfg).expect("email configured");
    /// assert_eq!(client.email, "ops@example.com");
    /// ```
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

    /// Sends an email notification (via the system `sendmail`) containing the provided hostname and IP.
    ///
    /// The email is sent to the client recipient configured in `self.email` and includes the
    /// package name and the client's configured hostname in the message body and subject.
    ///
    /// # Errors
    ///
    /// Returns an `Err` if spawning `sendmail` fails, writing to `sendmail`'s stdin fails,
    /// waiting for the `sendmail` process fails, or if `sendmail` exits with a non-success status.
    /// The error message includes any captured `sendmail` stderr where available.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::IpAddr;
    ///
    /// let client = crate::clients::emailonly::EmailonlyClient {
    ///     email: "ops@example.com".to_string(),
    ///     hostname: "sender-host".to_string(),
    /// };
    /// let ip: IpAddr = "127.0.0.1".parse().unwrap();
    /// // This will attempt to invoke `sendmail` on the host running the test.
    /// let _ = client.send_email("example-host", ip);
    /// ```
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
    /// Send an email notification containing the given hostname and IP, without modifying DNS records.
    ///
    /// This sends an email to the client's configured recipient describing `hostname` and `ip`, and always
    /// returns success since no DNS update is performed.
    ///
    /// # Parameters
    ///
    /// - `hostname`: Hostname to include in the notification subject and body.
    /// - `ip`: IP address to include in the notification body.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success; an `Err` if sending the email fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::net::IpAddr;
    /// // construct an EmailonlyClient (fields shown for illustration; use the public constructor in real code)
    /// let client = EmailonlyClient { email: "ops@example.com".into(), hostname: "my-host".into() };
    /// let ip: IpAddr = "203.0.113.5".parse().unwrap();
    /// client.update_record("my-host", ip).unwrap();
    /// ```
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        log::info!("Email-only mode: sending notification for {} -> {}", hostname, ip);
        
        // Send email notification
        self.send_email(hostname, ip)?;
        
        // Always return success - we don't actually update DNS
        Ok(())
    }

    /// Validate that the client is configured to send email notifications and that a local MTA is available.
    ///
    /// Checks that a recipient email address is present and that the `sendmail` command can be invoked on the system.
    ///
    /// # Returns
    ///
    /// `Ok(())` if an email address is configured and `sendmail` is available, `Err` with a descriptive message otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = EmailonlyClient { email: "ops@example.com".into(), hostname: "host1".into() };
    /// assert!(client.validate_config().is_ok());
    /// ```
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

    /// Provider identifier for this client.
    ///
    /// Returns the provider name `"EmailOnly"`.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = EmailonlyClient { email: String::from("recipient@example.com"), hostname: String::from("host") };
    /// assert_eq!(client.provider_name(), "EmailOnly");
    /// ```
    fn provider_name(&self) -> &str {
        "EmailOnly"
    }
}