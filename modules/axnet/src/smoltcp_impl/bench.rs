use super::{AxNetRxToken, AxNetTxToken, STANDARD_MTU};
use super::{DeviceWrapper, InterfaceWrapper};
use smoltcp::phy::{Device, RxToken, TxToken};

const GB: usize = 1000 * MB;
const MB: usize = 1000 * KB;
const KB: usize = 1000;

#[allow(unused)]
impl DeviceWrapper {
    pub fn bench_transmit_bandwidth(&mut self) {
        // 10 Gb
        const MAX_SEND_BYTES: usize = 10 * GB;
        let mut send_bytes: usize = 0;
        let mut past_send_bytes: usize = 0;
        let mut past_time = InterfaceWrapper::current_time();

        // Send bytes
        while send_bytes < MAX_SEND_BYTES {
            if let Some(tx_token) = self.transmit(InterfaceWrapper::current_time()) {
                AxNetTxToken::consume(tx_token, STANDARD_MTU, |tx_buf| {
                    tx_buf[0..12].fill(1);
                    // ether type: IPv4
                    tx_buf[12..14].copy_from_slice(&[0x08, 0x00]);
                    tx_buf[14..STANDARD_MTU].fill(1);
                });
                send_bytes += STANDARD_MTU;
            }

            let current_time = InterfaceWrapper::current_time();
            if (current_time - past_time).secs() == 1 {
                let gb = ((send_bytes - past_send_bytes) * 8) / GB;
                let mb = (((send_bytes - past_send_bytes) * 8) % GB) / MB;
                let gib = (send_bytes - past_send_bytes) / GB;
                let mib = ((send_bytes - past_send_bytes) % GB) / MB;
                info!(
                    "Transmit: {}.{:03}GBytes, Bandwidth: {}.{:03}Gbits/sec.",
                    gib, mib, gb, mb
                );
                past_time = current_time;
                past_send_bytes = send_bytes;
            }
        }
    }

    pub fn bench_receive_bandwidth(&mut self) {
        // 10 Gb
        const MAX_RECEIVE_BYTES: usize = 10 * GB;
        let mut receive_bytes: usize = 0;
        let mut past_receive_bytes: usize = 0;
        let mut past_time = InterfaceWrapper::current_time();
        // Receive bytes
        while receive_bytes < MAX_RECEIVE_BYTES {
            if let Some(rx_token) = self.receive(InterfaceWrapper::current_time()) {
                AxNetRxToken::consume(rx_token.0, |rx_buf| {
                    receive_bytes += rx_buf.len();
                });
            }

            let current_time = InterfaceWrapper::current_time();
            if (current_time - past_time).secs() == 1 {
                let gb = ((receive_bytes - past_receive_bytes) * 8) / GB;
                let mb = (((receive_bytes - past_receive_bytes) * 8) % GB) / MB;
                let gib = (receive_bytes - past_receive_bytes) / GB;
                let mib = ((receive_bytes - past_receive_bytes) % GB) / MB;
                info!(
                    "Receive: {}.{:03}GBytes, Bandwidth: {}.{:03}Gbits/sec.",
                    gib, mib, gb, mb
                );
                past_time = current_time;
                past_receive_bytes = receive_bytes;
            }
        }
    }
}
