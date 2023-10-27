#!/bin/bash
#
# Delete virtual interface (e.g. virtual bridge).
#
# sudo ./del-iface.sh <iface>

IFACE=$1

if [ -z "$IFACE" ]; then
    echo "Usage: $0 <iface>"
    exit 1
fi

ip link del $IFACE
