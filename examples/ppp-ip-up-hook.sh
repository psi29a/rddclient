#!/bin/bash
######################################################################
## rddclient hook for PPP connections
######################################################################
## Automatically updates DNS when PPP (Point-to-Point Protocol) 
## connection comes up. Useful for dial-up, PPPoE, PPTP, L2TP, etc.
##
## Installation:
##   1. Copy to: /etc/ppp/ip-up.local
##   2. Make executable: chmod +x /etc/ppp/ip-up.local
##   3. Restart pppd or reconnect
##
## Environment variables provided by pppd:
##   $1 (INTERFACE)    - Interface name (ppp0, ppp1, etc.)
##   $2 (TTY)          - Serial device (or pseudo-tty)
##   $3 (SPEED)        - Link speed
##   $4 (LOCAL_IP)     - Local IP address assigned
##   $5 (REMOTE_IP)    - Remote IP address (peer)
##   $6 (IPPARAM)      - Additional parameter from pppd config
##
## For more info: man pppd
######################################################################

INTERFACE=$1
TTY=$2
SPEED=$3
LOCAL_IP=$4
REMOTE_IP=$5
IPPARAM=$6

# log writes its arguments to the system logger using the tag "rddclient-ppp".
log() {
    logger -t rddclient-ppp "$@"
}

log "PPP connection up: $INTERFACE (local: $LOCAL_IP, remote: $REMOTE_IP)"

# Wait for network to stabilize
sleep 2

# Update DNS with the assigned IP
if [ -x /usr/local/bin/rddclient ]; then
    if [ -n "$LOCAL_IP" ]; then
        log "Updating DNS with IP: $LOCAL_IP"
        /usr/local/bin/rddclient --ip "$LOCAL_IP" 2>&1 | logger -t rddclient-ppp
    else
        log "Warning: No local IP detected, using auto-detection"
        /usr/local/bin/rddclient 2>&1 | logger -t rddclient-ppp
    fi
else
    log "Error: /usr/local/bin/rddclient not found"
fi

exit 0