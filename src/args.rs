use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(version, about = "Rust replacement for ddclient - Multi-provider Dynamic DNS updater", long_about = None)]
pub struct Args {
    /// DNS protocol/provider (cloudflare, dyndns2, namecheap, etc.) - ddclient compatible
    #[arg(long, default_value = "dyndns2")]
    pub protocol: String,

    /// Login/username for authentication - ddclient compatible
    #[arg(long)]
    pub login: Option<String>,

    /// Password (for basic auth providers)
    #[arg(long)]
    pub password: Option<String>,

    /// Server URL for the DNS provider - ddclient compatible
    #[arg(long)]
    pub server: Option<String>,

    /// DNS zone name (e.g., example.com) - ddclient compatible
    #[arg(long)]
    pub zone: Option<String>,

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

    /// Cache file path for storing state (default: /var/cache/rddclient/rddclient.cache) - ddclient compatible
    #[arg(long)]
    pub cache: Option<String>,

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

    /// Force DNS update even if IP hasn't changed - ddclient compatible
    #[arg(long, default_value = "false")]
    pub force: bool,

    /// IP detection method: 'ip', 'web', 'if', 'cmd' - ddclient compatible
    #[arg(long)]
    pub use_method: Option<String>,

    /// Network interface for IP detection (use with --use=if) - ddclient compatible
    #[arg(long)]
    pub if_name: Option<String>,

    /// Command to execute for IP detection (use with --use=cmd) - ddclient compatible
    #[arg(long)]
    pub cmd: Option<String>,

    /// Web service URL for IP detection (use with --use=web) - ddclient compatible
    #[arg(long)]
    pub web: Option<String>,

    /// Minimum interval between updates (e.g., 30s, 5m, 1h) - ddclient compatible
    #[arg(long)]
    pub min_interval: Option<String>,

    /// Maximum interval between forced updates (e.g., 25d, 30d) - ddclient compatible
    #[arg(long)]
    pub max_interval: Option<String>,

    /// Minimum interval between update attempts after an error (e.g., 5m, 10m) - ddclient compatible
    #[arg(long)]
    pub min_error_interval: Option<String>,
}

impl Args {
    pub fn new() -> Self {
        Self::parse()
    }
}
