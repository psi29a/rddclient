#!/bin/bash
######################################################################
## rddclient hook for NetworkManager dispatcher
######################################################################
## Automatically updates DNS when network connection changes.
##
## Installation:
##   1. Copy to: /etc/NetworkManager/dispatcher.d/99-rddclient
##   2. Make executable: chmod +x /etc/NetworkManager/dispatcher.d/99-rddclient
##   3. Restart NetworkManager: sudo systemctl restart NetworkManager
##
## Environment variables provided by NetworkManager:
##   $1 (INTERFACE)         - Interface name (eth0, wlan0, etc.)
##   $2 (ACTION)            - Event type (up, down, dhcp4-change, etc.)
##   $IP4_ADDRESS_0         - IPv4 address (if available)
##   $IP6_ADDRESS_0         - IPv6 address (if available)
##   $CONNECTION_ID         - Connection profile name
##
## For more info: man NetworkManager-dispatcher
######################################################################

INTERFACE=$1
ACTION=$2

# Log function
log() {
    logger -t rddclient-dispatcher "$@"
}

# Only run on these actions
case "$ACTION" in
    up|dhcp4-change|dhcp6-change)
        log "Network event: $ACTION on $INTERFACE (connection: $CONNECTION_ID)"
        
        # Wait a moment for network to stabilize
        sleep 2
        
        # Update DNS (let rddclient auto-detect IP)
        if [ -x /usr/local/bin/rddclient ]; then
            log "Triggering DNS update"
            /usr/local/bin/rddclient 2>&1 | logger -t rddclient-dispatcher
        else
            log "Error: /usr/local/bin/rddclient not found"
        fi
        ;;
    
    down)
        log "Network down on $INTERFACE, skipping DNS update"
        ;;
esac

exit 0
