#!/usr/bin/bash

echo "Setting up tap interface for qemu"
sudo ip tuntap add qemu-tap0 mode tap
sudo ip addr add 10.0.2.2/24 dev qemu-tap0
sudo ip link set up dev qemu-tap0
