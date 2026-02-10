// Copyright 2025 The Axvisor Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! AXFS - Axvisor Filesystem Module
#![cfg_attr(all(not(test), not(doc)), no_std)]
#[macro_use]
extern crate log;
extern crate alloc;

mod dev;
mod fs;
mod mounts;
mod partition;
mod root;

pub mod api;
pub mod fops;

use crate::partition::PartitionInfo;
use alloc::{
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};
use axdriver::{AxDeviceContainer, prelude::*};

/// Initializes filesystems by block devices.
pub fn init_filesystems(mut blk_devs: AxDeviceContainer<AxBlockDevice>, bootargs: Option<&str>) {
    info!("Initialize filesystems...");

    let dev = blk_devs.take_one().expect("No block device found!");
    info!("  use block device 0: {:?}", dev.device_name());
    let mut disk = self::dev::Disk::new(dev);

    // Parse root parameter from bootargs
    let root_spec = parse_root_spec(bootargs);

    // Try to scan GPT partitions first
    match self::partition::scan_gpt_partitions(&mut disk) {
        Ok(partitions) if !partitions.is_empty() => {
            initialize_with_partitions(disk, partitions, &root_spec)
        }
        Ok(_) => {
            warn!("No partitions found, mount ramfs as rootfs");
            self::root::init_rootfs_with_ramfs();
        }
        Err(e) => {
            warn!("Failed to scan GPT partitions: {:?}", e);
        }
    }
}

/// Initialize filesystems with detected partitions
fn initialize_with_partitions(
    disk: self::dev::Disk,
    partitions: Vec<PartitionInfo>,
    root_spec: &RootSpec,
) {
    info!(
        "Found {} partitions, initializing with dynamic filesystem detection",
        partitions.len()
    );

    // Find root partition based on specification
    let root_partition_index = find_root_partition(&partitions, root_spec);

    // Check if any partition has a supported filesystem
    let has_supported_fs = partitions.iter().any(|p| p.filesystem_type.is_some());

    if has_supported_fs {
        // Try to initialize with partitions
        let disk_arc = Arc::new(disk);
        if !self::root::init_rootfs_with_partitions(disk_arc, partitions, root_partition_index) {
            warn!("Failed to initialize with partitions.");
        }
    } else {
        warn!("No supported filesystem found in partitions.");
    }
}

/// Format a GUID as a string in format used by Linux PARTUUID
fn format_guid_as_partuuid(guid: &[u8; 16]) -> alloc::string::String {
    // Linux PARTUUID format is little-endian for first 3 fields,
    // and big-endian for the rest
    alloc::format!(
        "{:02X}{:02X}{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}",
        guid[3],
        guid[2],
        guid[1],
        guid[0], // First 4 bytes (little-endian)
        guid[5],
        guid[4], // Next 2 bytes (little-endian)
        guid[7],
        guid[6], // Next 2 bytes (little-endian)
        guid[8],
        guid[9], // Next 2 bytes (big-endian)
        guid[10],
        guid[11],
        guid[12],
        guid[13],
        guid[14],
        guid[15] // Last 6 bytes (big-endian)
    )
}

/// Root filesystem specification
#[derive(Debug, Default)]
struct RootSpec {
    partition_index: Option<usize>,
    partuuid: Option<String>,
    uuid: Option<String>,
    partlabel: Option<String>,
}

/// Parse root parameter from bootargs
fn parse_root_spec(bootargs: Option<&str>) -> RootSpec {
    let mut spec = RootSpec::default();

    if let Some(bootargs) = bootargs {
        if let Some(root_arg) = bootargs
            .split_whitespace()
            .find(|arg| arg.starts_with("root="))
        {
            let root_value = root_arg.strip_prefix("root=").unwrap_or("");

            spec = match root_value {
                v if v.starts_with("/dev/sda") => parse_device_path(v, "/dev/sda"),
                v if v.starts_with("/dev/mmcblk") => parse_mmcblk_path(v),
                v if v.starts_with("PARTUUID=") => {
                    let partuuid = v.strip_prefix("PARTUUID=").unwrap_or("").to_uppercase();
                    info!("Looking for partition with PARTUUID: {}", partuuid);
                    RootSpec {
                        partuuid: Some(partuuid),
                        ..Default::default()
                    }
                }
                v if v.starts_with("UUID=") => {
                    let uuid = v.strip_prefix("UUID=").unwrap_or("").to_uppercase();
                    info!("Looking for filesystem with UUID: {}", uuid);
                    RootSpec {
                        uuid: Some(uuid),
                        ..Default::default()
                    }
                }
                v if v.starts_with("PARTLABEL=") => {
                    let partlabel = v.strip_prefix("PARTLABEL=").unwrap_or("").to_string();
                    info!("Looking for partition with PARTLABEL: {}", partlabel);
                    RootSpec {
                        partlabel: Some(partlabel),
                        ..Default::default()
                    }
                }
                _ => spec,
            };
        }
    }

    spec
}

/// Parse device path like /dev/sdaX
fn parse_device_path(path: &str, prefix: &str) -> RootSpec {
    if let Some(part_num) = path.strip_prefix(prefix) {
        if let Ok(num) = part_num.parse::<usize>() {
            if num > 0 {
                return RootSpec {
                    partition_index: Some(num - 1),
                    ..Default::default()
                };
            }
        }
    }
    RootSpec::default()
}

/// Parse mmcblk path like /dev/mmcblkXpY
fn parse_mmcblk_path(path: &str) -> RootSpec {
    if let Some(remaining) = path.strip_prefix("/dev/mmcblk") {
        if let Some(p_pos) = remaining.find('p') {
            let part_str = &remaining[p_pos + 1..];
            if let Ok(num) = part_str.parse::<usize>() {
                if num > 0 {
                    return RootSpec {
                        partition_index: Some(num - 1),
                        ..Default::default()
                    };
                }
            }
        }
    }
    RootSpec::default()
}

/// Find partition index based on root specification
fn find_root_partition(partitions: &[PartitionInfo], root_spec: &RootSpec) -> Option<usize> {
    // If we have a specific partition index, use it
    if let Some(index) = root_spec.partition_index {
        return if index < partitions.len() {
            Some(index)
        } else {
            None
        };
    }

    // Try to match by PARTUUID or UUID
    for (i, partition) in partitions.iter().enumerate() {
        if partition.filesystem_type.is_none() {
            continue;
        }

        // Check PARTUUID match
        if let Some(ref partuuid) = root_spec.partuuid {
            let partition_guid = format_guid_as_partuuid(&partition.unique_partition_guid);
            debug!("Partition {} PARTUUID: {}", i, partition_guid);
            if partition_guid.contains(partuuid) {
                info!("Found matching partition by PARTUUID: {}", i);
                return Some(i);
            }
        }

        // Check UUID match
        if let Some(ref uuid) = root_spec.uuid {
            if let Some(ref partition_uuid) = partition.filesystem_uuid {
                if partition_uuid.to_uppercase() == *uuid {
                    info!("UUID matches partition {} ({})", i, partition.name);
                    return Some(i);
                }
            }
        }

        // Check PARTLABEL match
        if let Some(ref partlabel) = root_spec.partlabel {
            if partition.name == *partlabel {
                info!("PARTLABEL matches partition {} ({})", i, partition.name);
                return Some(i);
            }
        }
    }

    return None;
}
