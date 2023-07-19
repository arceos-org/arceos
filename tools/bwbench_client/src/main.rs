//! A raw socket benchmark client.

#![deny(warnings)]
#![deny(missing_docs)]
#![allow(dead_code, unused_variables)]

use crate::device::NetDevice;
use chrono::Local;
use std::env;
use std::fmt::Display;

mod device;

const STANDARD_MTU: usize = 1500;

const MAX_BYTES: usize = 10 * GB;
const MB: usize = 1000 * 1000;
const GB: usize = 1000 * MB;

struct EthernetMacAddress([u8; 6]);

impl Display for EthernetMacAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mac = self.0;
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
        )
    }
}

enum Client {
    Sender,
    Receiver,
}

fn transmit_benchmark(interface: &str) {
    println!("Sender Mode!");
    let mut dev = NetDevice::new(interface).unwrap();

    let mut tx_buf = [1u8; STANDARD_MTU];
    // ether type: IPv4
    tx_buf[12..14].copy_from_slice(&[0x08, 0x00]);

    let mut send_bytes = 0;
    let mut past_send_bytes = 0;
    let mut past_time = Local::now();

    loop {
        if let Ok(len) = dev.send(&tx_buf[..]) {
            send_bytes += len;
            let current_time = Local::now();
            if current_time.signed_duration_since(past_time).num_seconds() == 1 {
                let gb = ((send_bytes - past_send_bytes) * 8) / GB;
                let mb = (((send_bytes - past_send_bytes) * 8) % GB) / MB;
                let gib = (send_bytes - past_send_bytes) / GB;
                let mib = ((send_bytes - past_send_bytes) % GB) / MB;
                println!(
                    "Transfer: {}.{:03}GBytes, Bandwidth: {}.{:03}Gbits/sec.",
                    gib, mib, gb, mb
                );
                past_send_bytes = send_bytes;
                past_time = current_time;
            }
        }

        if send_bytes >= MAX_BYTES {
            break;
        }
    }
}

fn receive_benchmark(interface: &str) {
    println!("Receiver Mode!");
    let mut dev = NetDevice::new(interface).unwrap();

    let mut receive_bytes = 0;
    let mut past_receive_bytes = 0;
    let mut past_time = Local::now();

    let mut rx_buffer = [0; STANDARD_MTU];

    loop {
        if let Ok(len) = dev.recv(&mut rx_buffer) {
            receive_bytes += len;
        }

        let current_time = Local::now();
        if current_time.signed_duration_since(past_time).num_seconds() == 1 {
            let gb = ((receive_bytes - past_receive_bytes) * 8) / GB;
            let mb = (((receive_bytes - past_receive_bytes) * 8) % GB) / MB;
            let gib = (receive_bytes - past_receive_bytes) / GB;
            let mib = ((receive_bytes - past_receive_bytes) % GB) / MB;
            println!(
                "Receive: {}.{:03}GBytes, Bandwidth: {}.{:03}Gbits/sec.",
                gib, mib, gb, mb
            );
            past_receive_bytes = receive_bytes;
            past_time = current_time;
        }

        if receive_bytes >= MAX_BYTES {
            break;
        }
    }
}

fn benchmark_bandwidth(client: Client, interface: &str) {
    match client {
        Client::Sender => transmit_benchmark(interface),
        Client::Receiver => receive_benchmark(interface),
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        panic!("Usage: cargo run --release [send|receive] <interface>");
    }

    let kind = args[1].as_str();
    let client = match kind.chars().next().unwrap() {
        's' => Client::Sender,
        'r' => Client::Receiver,
        _ => panic!("Unknown Mode!"),
    };

    let interface = args[2].as_str();

    benchmark_bandwidth(client, interface);
}
