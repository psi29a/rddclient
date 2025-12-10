#!/bin/bash
######################################################################
## rddclient hook for dhclient (DHCP client)
######################################################################
## Automatically updates DNS when DHCP lease is obtained/renewed.
##
## Installation (Debian/Ubuntu):
##   1. Copy to: /etc/dhcp/dhclient-exit-hooks.d/rddclient
##   2. Make executable: chmod +x /etc/dhcp/dhclient-exit-hooks.d/rddclient
##   3. Restart dhclient or renew lease: sudo dhclient -r && sudo dhclient
##
## Installation (Red Hat/Fedora):
##   1. Copy to: /etc/dhcp/dhclient-exit-hooks
##   2. Make executable: chmod +x /etc/dhcp/dhclient-exit-hooks
##
## Environment variables provided by dhclient:
##   $reason     - Why dhclient is running (BOUND, RENEW, REBIND, etc.)
##   $new_ip_address - New IP address assigned
##   $old_ip_address - Previous IP address (if any)
##
######################################################################

# Only run on IP address changes
case "$reason" in
    BOUND|RENEW|REBIND|REBOOT)
        # Check if IP address actually changed
        if [ -n "$new_ip_address" ] && [ "$new_ip_address" != "$old_ip_address" ]; then
            logger -t rddclient "DHCP IP change detected: $old_ip_address -> $new_ip_address"
            
            # Update DNS with new IP
            if [ -x /usr/local/bin/rddclient ]; then
                /usr/local/bin/rddclient --ip "$new_ip_address" 2>&1 | logger -t rddclient
            else
                logger -t rddclient "Error: /usr/local/bin/rddclient not found"
            fi
        fi
        ;;
esac
