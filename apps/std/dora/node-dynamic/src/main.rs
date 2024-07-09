use dora_node_api::dora_core::config::{DataId, NodeId};
use dora_node_api::Event;
use dora_node_api::{self, arrow::array::UInt64Array, DoraNode};
use rand::Rng;
// use std::time::Duration;
use std::net::{IpAddr, Ipv4Addr};
use uhlc::system_time_clock;

#[no_mangle]
pub extern "C" fn ceil() {
    println!("ceil");
}

#[no_mangle]
pub extern "C" fn sqrt() {
    println!("sqrt");
}

static REMOTE_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(10, 0, 2, 2));

fn main() -> eyre::Result<()> {
    ceil();
    sqrt();
    println!(
        "Dora node-dynamic on ArceOS booted at time {}",
        chrono::Local::now()
    );

    let latency = DataId::from("latency".to_owned());
    let _throughput = DataId::from("throughput".to_owned());

    let (mut node, mut events) = DoraNode::init_from_node_id(
        NodeId::from("rust-node-dynamic".to_string()),
        Some(REMOTE_IP),
    )?;

    let sizes = [1, 10 * 512, 100 * 512];

    // test latency first
    for size in sizes {
        for i in 0..100 {
            if let Some(event) = events.recv() {
                // println!("node recv event[{}] {:#?}", i, event);
                match event {
                    Event::Input {
                        id: _,
                        data: _,
                        metadata,
                    } => {
                        let mut random_data: Vec<u64> = rand::thread_rng()
                            .sample_iter(rand::distributions::Standard)
                            .take(size)
                            .collect();
                        let t_send = system_time_clock().as_u64();
                        let beginning_slice = random_data.get_mut(0).unwrap();
                        *beginning_slice = t_send;

                        let random_data: UInt64Array = random_data.into();

                        node.send_output(latency.clone(), metadata.parameters, random_data)?;
                    }
                    _ => {}
                }
            } else {
                break;
            }
        }
    }

    Ok(())
}
