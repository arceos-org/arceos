#!/bin/bash

HOST_IF=$1

if [ -z "$HOST_IF" ]; then
    echo "Usage: $0 <host_interface>"
    exit 1
fi

echo "Setting up tap interface for QEMU"

ip tuntap add tap0 mode tap
ip addr add 10.0.2.2/24 dev tap0
ip link set up dev tap0

sysctl -w net.ipv4.ip_forward=1
iptables -t nat -A POSTROUTING -s 10.0.2.0/24 -o ${HOST_IF} -j MASQUERADE
