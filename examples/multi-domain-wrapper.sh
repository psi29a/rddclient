#!/bin/bash
######################################################################
## rddclient wrapper for multiple domains
######################################################################
## This wrapper allows running rddclient with multiple configuration
## files, useful when managing different domains with different credentials.
##
## Usage:
##   ./multi-domain-wrapper.sh [primary-ip]
##
## Setup:
##   1. Create /etc/rddclient/rddclient.conf (primary domain)
##   2. Create /etc/rddclient/domain2.conf (secondary domain)
##   3. Configure this script's CONFIGS array
##   4. Make executable: chmod +x multi-domain-wrapper.sh
##
######################################################################

set -e

# Configuration files to process
CONFIGS=(
    "/etc/rddclient/rddclient.conf"
    "/etc/rddclient/domain2.conf"
    "/etc/rddclient/domain3.conf"
)

# Optional: IP address passed as argument (from network hook, etc.)
IP=${1:-}

# Path to rddclient binary
RDDCLIENT=${RDDCLIENT:-/usr/local/bin/rddclient}

# Verify rddclient exists
if [ ! -x "$RDDCLIENT" ]; then
    echo "Error: rddclient not found at $RDDCLIENT" >&2
    exit 1
fi

# Process each configuration
for config in "${CONFIGS[@]}"; do
    if [ -f "$config" ]; then
        echo "Updating DNS with config: $config"
        
        if [ -n "$IP" ]; then
            # Use provided IP
            "$RDDCLIENT" --file "$config" --ip "$IP" || echo "Warning: Update failed for $config"
        else
            # Auto-detect IP
            "$RDDCLIENT" --file "$config" || echo "Warning: Update failed for $config"
        fi
    else
        echo "Warning: Config file not found: $config" >&2
    fi
done

echo "Multi-domain update complete"
