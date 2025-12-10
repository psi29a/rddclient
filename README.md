# rddclient - Rust Dynamic DNS Client

A fast, lightweight Dynamic DNS updater written in Rust that supports multiple DNS providers.

## Supported Providers

- ✅ **[1984.is](https://www.1984.is)** - DynDNS2-compatible protocol
- ✅ **[Afraid.org](https://freedns.afraid.org)** - Token-based update API (v2)
- ✅ **[Cloudflare](https://www.cloudflare.com)** - Full API support with zone management
- ✅ **[ChangeIP](https://www.changeip.com)** - Legacy JSON protocol with basic auth
- ✅ **[ClouDNS](https://www.cloudns.net)** - Simple dynurl-based updates
- ✅ **[CloudXNS](https://www.cloudxns.net)** - REST API with API key/secret
- ✅ **[DDNS.FM](https://ddns.fm)** - DDNS service with REST API
- ✅ **[DDNSS](https://www.ddnss.de)** - Simple token-based GET protocol
- ✅ **[deSEC](https://desec.io)** - German DNS with token auth (DynDNS2-compatible)
- ✅ **[DigitalOcean](https://www.digitalocean.com/)** - REST API with token authentication
- ✅ **[Dinahosting](https://dinahosting.com)** - REST API with basic auth
- ✅ **[Directnic](https://www.directnic.com)** - Pre-configured URL updates
- ✅ **[DNS Made Easy](https://dnsmadeeasy.com)** - Dynamic DNS endpoint
- ✅ **[DNSExit2](https://www.dnsexit.com)** - JSON API v2 with API key
- ✅ **[DNSPod](https://www.dnspod.cn)** - Chinese DNS with token-based API
- ✅ **[Domeneshop](https://api.domeneshop.no/docs/#tag/ddns/paths/~1dyndns~1update/get)** - REST API with basic auth
- ✅ **[DonDominio](https://www.dondominio.com)** - JSON API with key auth
- ✅ **[DSLReports](https://www.dslreports.com)** - DSLReports legacy protocol
- ✅ **[DuckDNS](https://duckdns.org)** - Simple token-based updates
- ✅ **DynDNS v1** - Legacy DynDNS protocol (pre-DynDNS2)
- ✅ **DynDNS2** - Compatible with [DynDNS](https://account.dyn.com), DNSdynamic, and other DynDNS2-compatible services
- ✅ **[Dynu](https://www.dynu.com)** - DynDNS2-compatible protocol
- ✅ **[EasyDNS](https://www.easydns.com)** - REST API with basic auth (10min update interval)
- ✅ **Email Only** - Send notifications via email instead of updating DNS (requires system sendmail)
- ✅ **[Enom](https://www.enom.com)** - Dynamic DNS API
- ✅ **[Freedns](https://freedns.afraid.org)** (afraid.org) - Hash-based update protocol
- ✅ **[Freemyip](https://freemyip.com)** - Simple token-based updates
- ✅ **[Gandi](https://gandi.net)** - REST API with API key
- ✅ **[GoDaddy](https://www.godaddy.com)** - REST API with key/secret
- ✅ **[Google Domains](https://domains.google.com)** - DynDNS2-compatible protocol
- ✅ **[Hetzner](https://www.hetzner.com)** - REST API with API token
- ✅ **[Hurricane Electric](https://dns.he.net)** (HE.net) - Simple update protocol
- ✅ **[Infomaniak](https://www.infomaniak.com)** - DynDNS2-compatible protocol
- ✅ **[INWX](https://www.inwx.com/)** - DynDNS2-compatible protocol
- ✅ **[Key-Systems](https://www.key-systems.net)** (RRPproxy) - Token-based updates
- ✅ **[Linode](https://www.linode.com)** - Linode API v4 with token auth
- ✅ **[Loopia](https://www.loopia.com)** - DynDNS2-compatible protocol
- ✅ **[LuaDNS](https://luadns.com)** - REST API with email/token auth
- ✅ **[Mythic Beasts](https://www.mythic-beasts.com)** - Modern dual-endpoint API
- ✅ **[Namecheap](https://www.namecheap.com)** - Native Dynamic DNS support
- ✅ **[NFSN](https://www.nearlyfreespeech.net)** (NearlyFreeSpeech.NET) - Basic auth updates
- ✅ **[Njalla](https://njal.la/docs/ddns)** - Simple API with password auth
- ✅ **[No-IP](https://www.noip.com)** - DynDNS2-compatible with No-IP specifics
- ✅ **nsupdate** - RFC 2136 Dynamic DNS Update protocol (requires DNS library)
- ✅ **[OVH](https://www.ovhcloud.com)** - REST API (simplified, requires proper signing for production)
- ✅ **[Porkbun](https://porkbun.com)** - REST API with key/secret
- ✅ **[Regfish](https://www.regfish.de)** - DynDNS2-compatible protocol
- ✅ **[Selfhost.de](https://www.selfhost.de)** - German provider with DynDNS2 protocol
- ✅ **[Sitelutions](https://www.sitelutions.com)** - DynDNS2-compatible protocol
- ✅ **[Woima.fi](https://www.woima.fi)** - Finnish DNS with DynDNS2 protocol
- ✅ **[Yandex](https://yandex.com)** - Yandex PDD API
- ✅ **[Zoneedit](https://www.zoneedit.com)** - DynDNS2-compatible protocol
- ✅ **ZoneEdit v1** - ZoneEdit legacy protocol

### DynDNS2-Compatible Providers

The `dyndns2` provider also works with many other services that support the DynDNS2 protocol, including but not limited to: DNSdynamic, DuckDNS (alternative), many router DDNS services, and custom DDNS implementations.

## Features

- 🚀 **Blazingly fast** - Compiled Rust vs interpreted Perl 
- 📦 **Tiny binary** - ~1.2MB vs ddclient's 200KB+ Perl script + dependencies
- 🎯 **Drop-in replacement** - Compatible with ddclient workflows and patterns
- 📝 **Flexible configuration** - File or command-line arguments
- 🌐 **Smart IP detection** - Automatic IP detection with multiple fallback sources
- ⚙️ **Easily extensible** - Clean architecture for adding new providers

## Installation

### From Source

```bash
cargo build --release
sudo cp target/release/rddclient /usr/local/bin/
```

The binary will be available at `target/release/rddclient`.

### System Integration

```bash
# Create config directory
sudo mkdir -p /etc/rddclient

# Copy example config
cp config.ini.example /etc/rddclient/rddclient.conf

# Set up systemd timer (recommended) or cron job
```

## Quick Start

### Command Line Examples

```bash
# Cloudflare (with API token)
rddclient --protocol cloudflare \
  --zone example.com \
  --login token \
  --password YOUR_API_TOKEN \
  --host ddns.example.com

# Cloudflare (with global API key)
rddclient --protocol cloudflare \
  --zone example.com \
  --login your-email@example.com \
  --password YOUR_GLOBAL_API_KEY \
  --host ddns.example.com

# DuckDNS
rddclient --protocol duckdns \
  --password YOUR_TOKEN \
  --host myhost

# GoDaddy
rddclient --protocol godaddy \
  --login YOUR_API_KEY \
  --password YOUR_API_SECRET \
  --host ddns.example.com

# No-IP
rddclient --protocol noip \
  --login YOUR_USERNAME \
  --password YOUR_PASSWORD \
  --host ddns.example.com

# Hurricane Electric
rddclient --protocol he \
  --password YOUR_KEY \
  --host ddns.example.com

# Use config file instead
rddclient --file /etc/rddclient/rddclient.conf
```

## Configuration Files

Create a `rddclient.conf` file for your provider (compatible with ddclient config format):

### Cloudflare

```ini
protocol = "cloudflare"
zone = "example.com"
login = "token"
password = "your_api_token_here"
host = "ddns.example.com"
ttl = 300
```

### DigitalOcean

```ini
protocol = "digitalocean"
password = "your_api_token_here"
host = "ddns.example.com"
```

### DuckDNS

```ini
protocol = "duckdns"
password = "your_token_here"
host = "myhost"  # without .duckdns.org
```

### DynDNS2 / No-IP / Generic

```ini
protocol = "dyndns2"  # or "noip"
username = "your_username"
password = "your_password"
host = "ddns.example.com"
# Optional custom server:
# server = "https://dynupdate.no-ip.com"
```

### Enom

```ini
protocol = "enom"
password = "your_update_token"
host = "ddns.example.com"
# Optional custom server:
# server = "https://dynamic.name-services.com"
```

### Freedns (afraid.org)

```ini
protocol = "freedns"
password = "your_update_token"  # unique per host
host = "ddns.example.com"
```

### Freemyip

```ini
protocol = "freemyip"
password = "your_token"
host = "ddns.example.com"
# Optional custom server:
# server = "https://freemyip.com"
```

### Gandi

```ini
protocol = "gandi"
password = "your_api_key_here"
host = "ddns.example.com"
```

### GoDaddy

```ini
protocol = "godaddy"
username = "your_api_key"
password = "your_api_secret"
host = "ddns.example.com"
```

### Google Domains

```ini
protocol = "googledomains"  # also: google-domains
username = "your_username"
password = "your_password"
host = "ddns.example.com"
# Optional custom server:
# server = "https://domains.google.com"
```

### Hurricane Electric (HE.net)

```ini
protocol = "he"
password = "your_update_key"
host = "ddns.example.com"
```

### Namecheap

```ini
protocol = "namecheap"
username = "example.com"  # your domain
password = "your_ddns_password"
host = "ddns.example.com"
```

### Porkbun

```ini
protocol = "porkbun"
username = "your_api_key"
password = "your_secret_key"
host = "ddns.example.com"
```

### Zoneedit

```ini
protocol = "zoneedit"
username = "your_username"
password = "your_password"
host = "ddns.example.com"
```

### 1984.is

```ini
protocol = "1984"  # also: one984
username = "your_username"
password = "your_password"
host = "ddns.example.com"
# Optional custom server:
# server = "https://www.1984.is"
```

### ChangeIP

```ini
protocol = "changeip"
username = "your_username"
password = "your_password"
host = "ddns.example.com"
# Optional custom server:
# server = "nic.changeip.com"
```

### ClouDNS

```ini
protocol = "cloudns"
password = "https://ipv4.cloudns.net/api/dynamicURL/?q=YOUR_UNIQUE_URL"
host = "ddns.example.com"  # hostname embedded in URL
```

### DDNSS

```ini
protocol = "ddnss"
password = "your_token"
host = "ddns.example.com"
# Optional custom server:
# server = "https://www.ddnss.de"
```

### DNS Made Easy

```ini
protocol = "dnsmadeeasy"  # also: dns-made-easy
username = "your_username"
password = "your_password"
host = "ddns.example.com"
# Optional custom server:
# server = "https://cp.dnsmadeeasy.com"
```

### DigitalOcean

```ini
protocol = "digitalocean"ing"
username = "your_username"
password = "your_password"
host = "ddns.example.com"
# Optional custom server:
# server = "https://dinahosting.com"
```

### Directnic

```ini
protocol = "directnic"
# Pre-configured URLs from Directnic dashboard:
server = "https://www.directnic.com/dns/dynUpdateDDNS?host=...&token=..."  # IPv4 URL
password = "https://www.directnic.com/dns/dynUpdateDDNS?host=...&token=..."  # IPv6 URL
host = "ddns.example.com"
```

### DNSExit2

```ini
protocol = "dnsexit2"
password = "your_api_key"
host = "ddns.example.com"
# Optional zone (defaults to dns_record):
# zone_id = "example.com"
# Optional TTL (defaults to 5):
# ttl = 5
```

### Domeneshop

```ini
protocol = "domeneshop"
username = "your_api_token"
password = "your_api_secret"
host = "ddns.example.com"
```

### DonDominio

```ini
protocol = "dondominio"
username = "your_username"
password = "your_api_key"
host = "ddns.example.com"
# Optional custom server:
# server = "https://dondns.dondominio.com"
```

### Dynu

```ini
protocol = "dynu"
username = "your_username"
password = "your_password"  # or API key
host = "ddns.example.com"
# Optional custom server:
# server = "https://api.dynu.com"
```

### EasyDNS

```ini
protocol = "easydns"
username = "your_username"
password = "your_password"
host = "ddns.example.com"
# Note: EasyDNS requires 10 minutes between updates
```

### Email Only

```ini
protocol = "emailonly"
email = "admin@example.com"
host = "ddns.example.com"
# Note: This sends email notifications instead of updating DNS
# Requires system sendmail to be installed and configured
```

### Hetzner

```ini
protocol = "hetzner"
password = "your_api_token"
zone = "example.com"
host = "ddns.example.com"
```

### Infomaniak

```ini
protocol = "infomaniak"
username = "your_username"
password = "your_password"
host = "ddns.example.com"
# Optional custom server:
# server = "https://infomaniak.com"
```

### INWX

```ini
protocol = "inwx"
username = "your_username"
password = "your_password"
host = "ddns.example.com"
```

### Loopia

```ini
protocol = "loopia"
username = "your_username"
password = "your_password"
host = "ddns.example.com"
# Optional custom server:
# server = "https://dns.loopia.se"
```

### Mythic Beasts

```ini
protocol = "mythicbeasts"  # also: mythic-beasts, mythicdyn
username = "your_username"
password = "your_password"
host = "ddns.example.com"
# Optional custom API server:
# server = "api.mythic-beasts.com"
```

### Njalla

```ini
protocol = "njalla"
password = "your_api_key"
host = "ddns.example.com"
```

### Regfish

```ini
protocol = "regfish"
username = "your_username"
password = "your_password"
host = "ddns.example.com"
# Optional custom server:
# server = "https://dyndns.regfish.de"
```

### Sitelutions

```ini
protocol = "sitelutions"
username = "your_username"
password = "your_password"
host = "ddns.example.com"
# Optional custom server:
# server = "https://www.sitelutions.com"
```

### Yandex

```ini
protocol = "yandex"
password = "your_pdd_token"
zone = "example.com"  # your domain
host = "ddns.example.com"
# Optional custom server:
# server = "https://pddimp.yandex.ru"
```

### nsupdate (RFC 2136)

```ini
protocol = "nsupdate"
server = "ns.example.com"
username = "zone_name"  # or TSIG key name
password = "tsig_key"
host = "ddns.example.com"
```

**Note:** Full nsupdate support requires a DNS protocol library. Consider using a dedicated nsupdate tool or DNS library for production use.

### CloudXNS

```ini
protocol = "cloudxns"
username = "your_api_key"
password = "your_secret_key"
host = "ddns.example.com"
# Optional custom server:
# server = "https://www.cloudxns.net"
```

### DNSPod

```ini
protocol = "dnspod"
password = "your_token_id,your_token"
host = "ddns.example.com"
# Optional custom server:
# server = "https://dnsapi.cn"
```

### Linode

```ini
protocol = "linode"
password = "your_api_token"
zone = "your_domain_id"
host = "your_record_id"
# Optional custom server:
# server = "https://api.linode.com"
```

### deSEC

```ini
protocol = "desec"
password = "your_token"
zone = "example.com"
host = "ddns.example.com"
# Optional custom server:
# server = "https://update.dedyn.io"
```

### LuaDNS

```ini
protocol = "luadns"
username = "your_email"
password = "your_api_token"
zone = "your_zone_id"
host = "your_record_id"
# Optional custom server:
# server = "https://api.luadns.com"
```

### NFSN (NearlyFreeSpeech.NET)

```ini
protocol = "nfsn"
username = "your_username"
password = "your_password"
host = "ddns.example.com"
# Optional custom server:
# server = "https://dynamicdns.park-your-domain.com"
```

### Afraid.org (v2)

```ini
protocol = "afraid"
password = "your_update_token"
host = "ddns.example.com"
# Optional custom server:
# server = "https://freedns.afraid.org"
```

### Woima.fi

```ini
protocol = "woima"
username = "your_username"
password = "your_password"
host = "ddns.example.com"
# Optional custom server:
# server = "https://www.woima.fi"
```

### Selfhost.de

```ini
protocol = "selfhost"
username = "your_username"
password = "your_password"
host = "ddns.example.com"
# Optional custom server:
# server = "https://carol.selfhost.de"
```

### DDNS.FM

```ini
protocol = "ddnsfm"  # also: ddns.fm
password = "your_token"
host = "ddns.example.com"
# Optional custom server:
# server = "https://api.ddns.fm"
```

### DSLReports

```ini
protocol = "dslreports"  # also: dslreports1
username = "your_username"
password = "your_password"
host = "ddns.example.com"
# Optional custom server:
# server = "https://www.dslreports.com"
```

### DynDNS v1 (Legacy)

```ini
protocol = "dyndns1"
username = "your_username"
password = "your_password"
host = "ddns.example.com"
# Optional custom server:
# server = "https://members.dyndns.org"
```

### Key-Systems (RRPproxy)

```ini
protocol = "keysystems"  # also: key-systems
password = "your_token"
host = "ddns.example.com"
# Optional custom server:
# server = "https://dynamicdns.key-systems.net"
```

### ZoneEdit v1 (Legacy)

```ini
protocol = "zoneedit1"
username = "your_username"
password = "your_password"
host = "ddns.example.com"
# Optional custom server:
# server = "https://dynamic.zoneedit.com"
```

Then run:

```bash
cloudflareddns --config cloudflareddns.ini
```

## Configuration Options

| Option | Description | Providers |
|--------|-------------|-----------|
| `--provider` | DNS provider name | All |
| `--zone-id` | Cloudflare Zone ID | Cloudflare |
| `--api-token` | API token | Cloudflare, DigitalOcean, Gandi, OVH |
| `--username` | Username or API key | DynDNS2, GoDaddy, Namecheap, No-IP, OVH, Porkbun, Zoneedit |
| `--password` | Password or secret | Most providers |
| `--server` | Custom API endpoint | DynDNS2, No-IP, others |
| `--dns-record` | DNS record(s) to update (comma-separated) | All |
| `--ttl` | TTL for DNS records | Cloudflare |
| `--ip` | Manually specify IP address | All |
| `--config` | Configuration file path | All |
| `--test` | Test mode (validate without updating) | All |
| `--verbose` | Verbose output | All |
| `--debug` | Debug output | All |

## Multiple DNS Records

Update multiple records at once:

```bash
cloudflareddns --provider cloudflare \
  --zone-id YOUR_ZONE_ID \
  --api-token YOUR_API_TOKEN \
  --dns-record "ddns.example.com,www.example.com,home.example.com"
```

## Automatic Updates

### Linux/macOS Cron

```bash
# Update every 5 minutes
*/5 * * * * /path/to/cloudflareddns --config /path/to/cloudflareddns.ini
```

### Systemd Service

Create `/etc/systemd/system/ddns-updater.service`:

```ini
[Unit]
Description=Dynamic DNS Updater
After=network-online.target

[Service]
Type=oneshot
ExecStart=/usr/local/bin/cloudflareddns --config /etc/cloudflareddns.ini
User=nobody
```

Create `/etc/systemd/system/ddns-updater.timer`:

```ini
[Unit]
Description=Run Dynamic DNS Updater every 5 minutes

[Timer]
OnBootSec=1min
OnUnitActiveSec=5min

[Install]
WantedBy=timers.target
```

Enable:

```bash
sudo systemctl enable ddns-updater.timer
sudo systemctl start ddns-updater.timer
```

## Provider-Specific Notes

### Cloudflare
- Requires Zone ID (found in domain Overview)
- API token needs `DNS:Edit` permissions

### DigitalOcean
- Personal Access Token required
- DNS record must already exist

### DuckDNS
- Free service, no account needed for basic use
- Token is per-account, works for all your domains
- Hostname should be without `.duckdns.org` suffix

### Freedns (afraid.org)
- Uses unique update token per hostname
- Token is different for each DNS record

### GoDaddy
- Requires API key/secret from developer.godaddy.com
- Production keys require domain ownership verification

### Hurricane Electric
- Free DNS hosting
- Update key is per-hostname (found in DNS management)

### OVH
- Current implementation is simplified
- Full production use requires proper API request signing
- Requires application key, secret, and consumer key

### Porkbun
- Requires API enabled in account settings
- Both API key and secret key needed

## Architecture

```
src/
├── main.rs              # Main application logic
├── args.rs              # CLI argument parsing
├── config.rs            # Configuration management
├── ip.rs                # IP address detection with fallback
└── clients/             # DNS provider implementations
    ├── mod.rs           # DnsClient trait and factory
    ├── cloudflare.rs
    ├── digitalocean.rs
    ├── duckdns.rs
    ├── dyndns2.rs
    ├── freedns.rs
    ├── gandi.rs
    ├── godaddy.rs
    ├── he.rs
    ├── namecheap.rs
    ├── noip.rs
    ├── ovh.rs
    ├── porkbun.rs
    └── zoneedit.rs
```

## Adding New Providers

To add a new DNS provider:

1. Create `src/clients/newprovider.rs`
2. Implement the `DnsClient` trait:
   - `update_record()` - Update DNS record
   - `validate_config()` - Validate configuration
   - `provider_name()` - Return provider name
3. Add module to `src/clients/mod.rs`
4. Add to `create_client()` factory function
5. Add configuration example

Example template:

```rust
use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

pub struct NewProviderClient {
    // provider-specific fields
}

impl NewProviderClient {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        // initialization
    }
}

impl DnsClient for NewProviderClient {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        // implementation
    }

    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        // validation
    }

    fn provider_name(&self) -> &str {
        "NewProvider"
    }
}
```

## Building for Release

```bash
cargo build --release
strip target/release/cloudflareddns  # Optional: reduce size further
```

## Troubleshooting

### IP Detection Issues
The application tries multiple IP detection services with automatic fallback. If all fail, specify the IP manually with `--ip`.

### Authentication Errors
- Double-check credentials in your configuration
- Ensure API tokens have correct permissions
- Check for trailing spaces in config file values

### DNS Update Failures
- Verify the DNS record exists (some providers require pre-creation)
- Check domain ownership in provider dashboard
- Enable `--verbose` or `--debug` for detailed logs

## License

GPLv3

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Roadmap

- [x] IPv6 support (Cloudflare implemented, other providers in progress)
- [ ] Additional providers (INWX, Hetzner, Njalla, etc.)
- [ ] Daemon mode with automatic interval updates
- [ ] Configuration validation command
- [ ] Web UI for configuration management
