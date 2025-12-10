# rddclient Examples

This directory contains example configurations and integration scripts for rddclient.

## Contents

### Deployment Integration
- `rddclient.service` - Systemd service unit for one-shot execution
- `rddclient.timer` - Systemd timer for periodic updates (every 5 minutes)
- `rddclient.cron` - Traditional cron job examples

### Network Hooks (automatic updates on IP change)
- `dhclient-exit-hook.sh` - DHCP client hook (Debian/Ubuntu/Red Hat)
- `networkmanager-dispatcher.sh` - NetworkManager dispatcher script
- `ppp-ip-up-hook.sh` - PPP connection hook (dial-up, PPPoE, PPTP, L2TP)

### Utilities
- `multi-domain-wrapper.sh` - Wrapper script for updating multiple domains

### Provider-Specific Configurations
- `rddclient.conf.example` - General configuration example (multi-provider)
- `cloudflare.conf` - Cloudflare DNS configuration example
- `duckdns.conf` - DuckDNS configuration example
- `noip.conf` - No-IP configuration example
- `namecheap.conf` - Namecheap Dynamic DNS configuration example

## Quick Start

### Systemd Timer (Recommended)

```bash
# 1. Install rddclient
sudo cp target/release/rddclient /usr/local/bin/
sudo chmod +x /usr/local/bin/rddclient

# 2. Create config directory and copy config
sudo mkdir -p /etc/rddclient
sudo cp examples/cloudflare.conf /etc/rddclient/rddclient.conf
sudo chmod 600 /etc/rddclient/rddclient.conf

# 3. Edit with your credentials
sudo nano /etc/rddclient/rddclient.conf

# 4. Install systemd units
sudo cp examples/rddclient.service /etc/systemd/system/
sudo cp examples/rddclient.timer /etc/systemd/system/

# 5. Enable and start
sudo systemctl daemon-reload
sudo systemctl enable rddclient.timer
sudo systemctl start rddclient.timer

# 6. Check status
sudo systemctl status rddclient.timer
sudo journalctl -u rddclient.service -f
```

### Cron Job

```bash
# Edit crontab
sudo crontab -e

# Add line (every 5 minutes)
*/5 * * * * /usr/local/bin/rddclient --file /etc/rddclient/rddclient.conf
```

### Network Hooks

```bash
# DHCP hook (automatic updates on IP change)
sudo cp examples/dhclient-exit-hook.sh /etc/dhcp/dhclient-exit-hooks.d/rddclient
sudo chmod +x /etc/dhcp/dhclient-exit-hooks.d/rddclient

# Or NetworkManager dispatcher
sudo cp examples/networkmanager-dispatcher.sh /etc/NetworkManager/dispatcher.d/99-rddclient
sudo chmod +x /etc/NetworkManager/dispatcher.d/99-rddclient
sudo systemctl restart NetworkManager
```

## Security Notes

**Important:** Configuration files contain sensitive credentials. Protect them:

```bash
sudo chmod 600 /etc/rddclient/rddclient.conf
sudo chown root:root /etc/rddclient/rddclient.conf
```

Never commit credentials to version control.

## See Also

- [Main README](../README.md) - General documentation
- [Parity Guide](../docs/parity.md) - Feature comparison with ddclient
- [Testing Guide](../docs/testing.md) - Testing documentation

