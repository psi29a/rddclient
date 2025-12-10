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
- 🎯 **Drop-in replacement** - Compatible with ddclient config format and workflows
- 🌍 **Full IPv6 support** - All 53 providers support both A and AAAA records
- 📝 **Flexible configuration** - ddclient-compatible config files or command-line arguments
- 🔄 **Smart IP detection** - Automatic IP detection with multiple fallback sources
- ⚙️ **Easily extensible** - Clean architecture for adding new providers

## Installation

### From Source

```bash
cargo build --release
sudo cp target/release/rddclient /usr/local/bin/
```

### System Integration

See the [`examples/`](examples/) directory for:
- Systemd service and timer units
- Cron job examples  
- Network hook scripts (DHCP, NetworkManager, PPP)
- Provider-specific configurations

## Quick Start

### Simple Example

```bash
# Cloudflare with API token
rddclient --protocol cloudflare \
  --zone example.com \
  --login token \
  --password YOUR_API_TOKEN \
  --host ddns.example.com

# Or use a config file (recommended)
rddclient --file /etc/rddclient/rddclient.conf
```

### Configuration File Example

```ini
# /etc/rddclient/rddclient.conf
protocol = cloudflare
zone = example.com
login = token
password = your_api_token_here
host = ddns.example.com
```

**For more examples**, see:
- [`examples/cloudflare.conf`](examples/cloudflare.conf) - Cloudflare setup
- [`examples/duckdns.conf`](examples/duckdns.conf) - DuckDNS setup  
- [`examples/noip.conf`](examples/noip.conf) - No-IP setup
- [`examples/namecheap.conf`](examples/namecheap.conf) - Namecheap setup
- [`examples/rddclient.conf.example`](examples/rddclient.conf.example) - Multi-provider template

## Documentation

- [`docs/parity.md`](docs/parity.md) - Feature parity with ddclient
- [`docs/testing.md`](docs/testing.md) - Testing strategy and coverage
- [`examples/README.md`](examples/README.md) - Deployment examples and integration

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

rddclient uses a modular architecture with provider-specific clients implementing a common `DnsClient` trait:

```
src/
├── main.rs              # Application entry point & orchestration
├── args.rs              # CLI argument parsing (Clap)
├── config.rs            # ddclient config file parser
├── ip.rs                # IP detection with fallback sources
└── clients/             # DNS provider implementations
    ├── mod.rs           # DnsClient trait & provider factory
    ├── cloudflare.rs    # Cloudflare API client
    ├── digitalocean.rs  # DigitalOcean API client
    ├── duckdns.rs       # DuckDNS API client
    ├── dyndns2.rs       # DynDNS2 protocol (40+ providers)
    └── ...              # 53 total providers
```

## Adding New Providers

To add a new DNS provider, see [`docs/adding-providers.md`](docs/adding-providers.md) for detailed instructions.

Quick overview:

1. Create `src/clients/newprovider.rs` implementing the `DnsClient` trait
2. Add module to `src/clients/mod.rs`
3. Add to `create_client()` factory function  
4. Add example configuration to `examples/`
5. Add tests

Example template:

```rust
use crate::clients::DnsClient;
use std::error::Error;
use std::net::IpAddr;

pub struct NewProviderClient {
    api_key: String,
    // provider-specific fields
}

impl DnsClient for NewProviderClient {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        // Make API call to update DNS record
        Ok(())
    }
}
```

## Troubleshooting

### IP Detection Issues
Automatic IP detection tries multiple sources with fallback. If all fail, specify manually with `--ip`.

### Authentication Errors
- Verify API credentials in configuration
- Check token/key permissions  
- Look for trailing spaces in config values

### DNS Update Failures
- Some providers require pre-creating DNS records
- Verify domain ownership in provider dashboard
- Use `--verbose` for detailed request/response logs

For more help, see [`docs/troubleshooting.md`](docs/troubleshooting.md).

## License

GPLv3

## Contributing

Contributions welcome! See provider guidelines in [`docs/ProviderGuidelines.md`](ddclient/docs/ProviderGuidelines.md) for adding new providers.

## Roadmap

See [`docs/parity.md`](docs/parity.md) for complete feature parity tracking with ddclient.

**Completed:**
- ✅ Full IPv6 support (all 53 providers)
- ✅ `--force` flag for forced updates
- ✅ ddclient config file compatibility

**In Progress:**
- State management & IP change detection
- Daemon mode with `--daemon` flag
- Rate limiting (`--min-interval`, `--max-interval`)

**Planned:**
- Email notifications
- Proxy support
- Advanced IP detection (`--use=if`, `--use=cmd`)

