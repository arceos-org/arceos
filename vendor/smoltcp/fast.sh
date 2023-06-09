#!/bin/bash

cargo batch \
--- build --no-default-features --features 'default' \
--- build --no-default-features --features 'std proto-ipv4' \
--- build --no-default-features --features 'std medium-ethernet phy-raw_socket proto-ipv6 socket-udp' \
--- build --no-default-features --features 'std medium-ethernet phy-tuntap_interface proto-ipv6 socket-udp' \
--- build --no-default-features --features 'std medium-ethernet proto-ipv4 proto-igmp socket-raw' \
--- build --no-default-features --features 'std medium-ethernet proto-ipv4 socket-udp socket-tcp' \
--- build --no-default-features --features 'std medium-ethernet proto-ipv4 socket-udp socket-tcp socket-dns' \
--- build --no-default-features --features 'std medium-ethernet proto-ipv4 proto-dhcpv4 socket-udp' \
--- build --no-default-features --features 'std medium-ethernet proto-ipv6 socket-udp' \
--- build --no-default-features --features 'std medium-ethernet proto-ipv6 socket-udp socket-dns' \
--- build --no-default-features --features 'std medium-ethernet proto-ipv6 socket-tcp' \
--- build --no-default-features --features 'std medium-ethernet proto-ipv4 socket-icmp socket-tcp' \
--- build --no-default-features --features 'std medium-ethernet proto-ipv6 socket-icmp socket-tcp' \
--- build --no-default-features --features 'std medium-ethernet proto-ipv4 socket-icmp socket-tcp socket-dns' \
--- build --no-default-features --features 'std medium-ethernet proto-ipv6 socket-icmp socket-tcp socket-dns' \
--- build --no-default-features --features 'std medium-ethernet proto-ipv4 proto-ipv6 socket-raw socket-udp socket-tcp socket-icmp async' \
--- build --no-default-features --features 'std medium-ethernet medium-ip proto-ipv4 proto-ipv6 socket-raw socket-udp socket-tcp socket-icmp async' \
--- build --no-default-features --features 'std medium-ip proto-ipv4 proto-ipv6 socket-raw socket-udp socket-tcp socket-icmp async' \
--- build --no-default-features --features 'std medium-ethernet medium-ip medium-ieee802154 proto-ipv6 socket-raw socket-udp socket-tcp socket-icmp async' \
--- build --no-default-features --features 'std medium-ethernet medium-ieee802154 proto-ipv6 socket-raw socket-udp socket-tcp socket-icmp async' \
--- build --no-default-features --features 'std medium-ip medium-ieee802154 proto-ipv6 socket-raw socket-udp socket-tcp socket-icmp async' \
--- build --no-default-features --features 'alloc medium-ethernet proto-ipv4 proto-ipv6 socket-raw socket-udp socket-tcp socket-icmp socket-dns' \
--- build --no-default-features --features 'medium-ip medium-ethernet medium-ieee802154 proto-ipv6 proto-ipv6 proto-igmp proto-dhcpv4 socket-raw socket-udp socket-tcp socket-icmp socket-dns async' \
--- build --no-default-features --features 'defmt medium-ip medium-ethernet proto-ipv6 proto-ipv6 proto-igmp proto-dhcpv4 socket-raw socket-udp socket-tcp socket-icmp socket-dns async' \
