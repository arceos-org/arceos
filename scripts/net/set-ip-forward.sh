#!/bin/bash
#
# Setup IP forwarding to allow Internet access in the VM.

WLAN_IF=$1
BR=virbr0
IP_RANGE=10.0.2.0/24

if [ -z "$WLAN_IF" ]; then
    echo "Usage: $0 <wlan_iface>"
    exit 1
fi

sysctl -w net.ipv4.ip_forward=1

iptables -t nat -A POSTROUTING -s $IP_RANGE -o $WLAN_IF -j MASQUERADE
iptables -A FORWARD -i $BR -j ACCEPT
iptables -A FORWARD -o $BR -m state --state RELATED,ESTABLISHED -j ACCEPT
