#!/bin/bash
#
# Bind a PCI device to the `vfio-pci` driver for PCI passthrough.
#
# Bind: sudo ./pci-bind.sh vfio-pci 02:00.0
# Unbind: sudo ./pci-bind.sh ixgbe 02:00.0

new_drv=$1
bdf=$2

if [ -z "$bdf" -o -z "$new_drv" ]; then
    echo "Usage: $0 <driver> <bus:dev.func>"
    exit 1
fi

bdf=0000:$bdf
old_drv=$(readlink /sys/bus/pci/devices/$bdf/driver | awk -F/ '{print $NF}')

echo "Bind $bdf from $old_drv to $new_drv"

echo $bdf > /sys/bus/pci/drivers/$old_drv/unbind
echo $new_drv > /sys/bus/pci/devices/$bdf/driver_override
echo $bdf > /sys/bus/pci/drivers/$new_drv/bind
