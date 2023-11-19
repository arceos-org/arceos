#!/bin/bash
#
# Disable IP forwarding and restore iptables rules.

WLAN_IF=$1
BR=virbr0
IP_RANGE=10.0.2.0/24

if [ -z "$WLAN_IF" ]; then
    echo "Usage: $0 <wlan_iface>"
    exit 1
fi

sysctl -w net.ipv4.ip_forward=0

iptables -t nat -D POSTROUTING -s $IP_RANGE -o $WLAN_IF -j MASQUERADE
iptables -D FORWARD -i $BR -j ACCEPT
iptables -D FORWARD -o $BR -m state --state RELATED,ESTABLISHED -j ACCEPT
