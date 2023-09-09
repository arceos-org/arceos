#!/bin/bash

HOST_IF=$1

if [ -z "$HOST_IF" ]; then
    echo "Usage: $0 <host_interface>"
    exit 1
fi

echo "Deleting tap interface for QEMU"

ip link del tap0
sysctl -w net.ipv4.ip_forward=0
iptables -t nat -D POSTROUTING -s 10.0.2.0/24 -o ${HOST_IF} -j MASQUERADE
