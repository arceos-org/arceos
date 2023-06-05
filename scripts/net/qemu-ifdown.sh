#!/usr/bin/bash

echo "Deleting tap interface for qemu"
sudo ip link del qemu-tap0
