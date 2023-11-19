#!/bin/bash
#
# Create a TAP interface and add it to the bridge.
#
# It's used for the startup script of QEMU netdev, DO NOT run it manually.

br=virbr0
if [ -n "$1" ]; then
    #create a TAP interface; qemu will handle it automatically.
    #tunctl -u $(whoami) -t $1
    #start up the TAP interface
    ip link set "$1" up
    brctl addif $br "$1"
    exit
else
    echo "Error: no interface specified"
    exit 1
fi
