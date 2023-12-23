#!/bin/bash
#
# Create virtual bridge for QEMU.
#
# sudo ./create-bridge.sh [virbr0]

BR=$1
IP=10.0.2.2

if [ -z "$BR" ]; then
    BR=virbr0
fi

echo "Deleting old virtual bridge $BR ..."

ip link set dev $BR down 2> /dev/null
brctl delbr $BR 2> /dev/null

echo "Setting up virtual bridge $BR ..."

brctl addbr $BR
ip addr add $IP/24 dev $BR
ip link set dev $BR up
brctl show $BR

ifconfig $BR
