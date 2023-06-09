#!/bin/bash

set -euxo pipefail

cargo test --no-default-features --features default
cargo test --no-default-features --features std,proto-ipv4
cargo test --no-default-features --features std,medium-ethernet,phy-raw_socket,proto-ipv6,socket-udp,socket-dns
cargo test --no-default-features --features std,medium-ethernet,phy-tuntap_interface,proto-ipv6,socket-udp
cargo test --no-default-features --features std,medium-ethernet,proto-ipv4,proto-ipv4-fragmentation,socket-raw,socket-dns
cargo test --no-default-features --features std,medium-ethernet,proto-ipv4,proto-igmp,socket-raw,socket-dns
cargo test --no-default-features --features std,medium-ethernet,proto-ipv4,socket-udp,socket-tcp,socket-dns
cargo test --no-default-features --features std,medium-ethernet,proto-ipv4,proto-dhcpv4,socket-udp
cargo test --no-default-features --features std,medium-ethernet,medium-ip,medium-ieee802154,proto-ipv6,socket-udp,socket-dns
cargo test --no-default-features --features std,medium-ethernet,proto-ipv6,socket-tcp
cargo test --no-default-features --features std,medium-ethernet,medium-ip,proto-ipv4,socket-icmp,socket-tcp
cargo test --no-default-features --features std,medium-ip,proto-ipv6,socket-icmp,socket-tcp
cargo test --no-default-features --features std,medium-ieee802154,proto-sixlowpan,socket-udp
cargo test --no-default-features --features std,medium-ieee802154,proto-sixlowpan,proto-sixlowpan-fragmentation,socket-udp
cargo test --no-default-features --features std,medium-ip,proto-ipv4,proto-ipv6,socket-tcp,socket-udp
cargo test --no-default-features --features std,medium-ethernet,medium-ip,medium-ieee802154,proto-ipv4,proto-ipv6,socket-raw,socket-udp,socket-tcp,socket-icmp,socket-dns,async
cargo check --no-default-features --features alloc,medium-ethernet,proto-ipv4,proto-ipv6,socket-raw,socket-udp,socket-tcp,socket-icmp
cargo check --no-default-features --features medium-ip,medium-ethernet,medium-ieee802154,proto-ipv6,proto-ipv6,proto-igmp,proto-dhcpv4,socket-raw,socket-udp,socket-tcp,socket-icmp,socket-dns,async
cargo check --no-default-features --features defmt,medium-ip,medium-ethernet,proto-ipv6,proto-ipv6,proto-igmp,proto-dhcpv4,socket-raw,socket-udp,socket-tcp,socket-icmp,socket-dns,async
cargo check --no-default-features --features defmt,alloc,medium-ip,medium-ethernet,proto-ipv6,proto-ipv6,proto-igmp,proto-dhcpv4,socket-raw,socket-udp,socket-tcp,socket-icmp,socket-dns,async

