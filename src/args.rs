use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(version, about = "Rust replacement for ddclient - Multi-provider Dynamic DNS updater", long_about = None)]
pub struct Args {
    /// DNS protocol/provider (cloudflare, dyndns2, namecheap, etc.) - ddclient compatible
    #[arg(long, default_value = "dyndns2")]
    pub protocol: String,

    /// Zone ID (Cloudflare only)
    #[arg(long)]
    pub zone_id: Option<String>,

    /// API token or password
    #[arg(long)]
    pub api_token: Option<String>,

    /// Login/username for authentication - ddclient compatible
    #[arg(long)]
    pub login: Option<String>,

    /// Password (for basic auth providers)
    #[arg(long)]
    pub password: Option<String>,

    /// Server/API endpoint (provider-specific)
    #[arg(long)]
    pub server: Option<String>,

    /// Hostname(s) to update (comma-separated) - ddclient compatible
    #[arg(long)]
    pub host: Option<String>,

    /// TTL for DNS records
    #[arg(long)]
    pub ttl: Option<u32>,

    /// Manually specify IP address (instead of auto-detection)
    #[arg(long)]
    pub ip: Option<String>,

    /// Configuration file path - ddclient compatible
    #[arg(long)]
    pub file: Option<String>,

    /// Test mode - validate config and show what would happen without updating (ddclient compatible)
    #[arg(long, default_value = "false")]
    pub test: bool,

    /// Verbose output
    #[arg(long, default_value = "false")]
    pub verbose: bool,

    /// Debug output
    #[arg(long, default_value = "false")]
    pub debug: bool,

    /// Quiet mode - suppress all output except errors
    #[arg(long, default_value = "false")]
    pub quiet: bool,
}

impl Args {
    pub fn new() -> Self {
        Self::parse()
    }
}
