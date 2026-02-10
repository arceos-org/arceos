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

//! Partition management and filesystem detection
//!
//! This module provides functionality to scan GPT partition tables and detect
//! filesystem types on each partition.

use alloc::{format, string::String, sync::Arc, vec, vec::Vec};
use axerrno::{AxResult, ax_err};
use axfs_vfs::VfsOps;
use log::{debug, info, warn};

use crate::dev::Disk;

/// Partition information
#[derive(Debug, Clone)]
pub struct PartitionInfo {
    /// Partition index (0-based)
    pub index: u32,
    /// Partition name
    pub name: String,
    /// Partition type GUID
    #[allow(dead_code)]
    pub partition_type_guid: [u8; 16],
    /// Unique partition GUID
    pub unique_partition_guid: [u8; 16],
    /// Filesystem UUID (if available)
    pub filesystem_uuid: Option<String>,
    /// Starting LBA
    pub starting_lba: u64,
    /// Ending LBA
    pub ending_lba: u64,
    /// Partition size in bytes
    pub size_bytes: u64,
    /// Detected filesystem type
    pub filesystem_type: Option<FilesystemType>,
}

/// Filesystem types that can be detected
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilesystemType {
    /// FAT32/FAT16 filesystem
    Fat,
    /// ext4/ext3/ext2 filesystem
    Ext4,
    /// Unknown filesystem
    Unknown,
}

/// GPT Header structure
#[repr(C, packed)]
struct GptHeader {
    signature: [u8; 8], // "EFI PART"
    revision: [u8; 4],
    header_size: [u8; 4],
    header_crc32: [u8; 4],
    reserved: [u8; 4],
    current_lba: [u8; 8],
    backup_lba: [u8; 8],
    first_usable_lba: [u8; 8],
    last_usable_lba: [u8; 8],
    disk_guid: [u8; 16],
    partition_entry_lba: [u8; 8],
    number_of_partition_entries: [u8; 4],
    size_of_partition_entry: [u8; 4],
    partition_entry_array_crc32: [u8; 4],
}

/// GPT Partition Entry structure
#[repr(C, packed)]
struct GptPartitionEntry {
    partition_type_guid: [u8; 16],
    unique_partition_guid: [u8; 16],
    starting_lba: [u8; 8],
    ending_lba: [u8; 8],
    attributes: [u8; 8],
    partition_name: [u16; 36], // UTF-16LE
}

/// GPT partition scanner
pub fn scan_gpt_partitions(disk: &mut Disk) -> AxResult<Vec<PartitionInfo>> {
    info!("Scanning for GPT partitions...");

    let disk_size = disk.size();
    if disk_size == 0 {
        return Ok(Vec::new());
    }

    // First, try to parse GPT partition table
    match parse_gpt_partitions(disk) {
        Ok(partitions) if !partitions.is_empty() => {
            info!("Found {} GPT partitions", partitions.len());
            return Ok(partitions);
        }
        Ok(_) => {
            info!("No GPT partitions found, trying MBR...");
        }
        Err(e) => {
            warn!("Failed to parse GPT: {:?}", e);
            info!("Trying MBR...");
        }
    }

    // If both GPT fail, treat the whole disk as a single partition
    warn!("No partition table found, treating whole disk as single partition");
    let filesystem_type = detect_filesystem_type(disk, 0);
    let partition = PartitionInfo {
        index: 0,
        name: String::from("disk"),
        partition_type_guid: [0; 16],
        unique_partition_guid: [0; 16],
        filesystem_uuid: None,
        starting_lba: 0,
        ending_lba: disk_size / 512,
        size_bytes: disk_size,
        filesystem_type,
    };

    Ok(vec![partition])
}

/// Parse GPT partition table
fn parse_gpt_partitions(disk: &mut Disk) -> AxResult<Vec<PartitionInfo>> {
    let mut partitions = Vec::new();

    // Read GPT Header from LBA 1
    let mut header_data = [0u8; 512];
    disk.set_position(512); // LBA 1
    if read_exact(disk, &mut header_data).is_err() {
        return ax_err!(InvalidData, "Failed to read GPT header");
    }

    // Check GPT signature
    if &header_data[0..8] != b"EFI PART" {
        return ax_err!(InvalidData, "Invalid GPT signature");
    }

    // Parse GPT header manually to avoid size mismatch
    let header = GptHeader {
        signature: header_data[0..8].try_into().unwrap(),
        revision: header_data[8..12].try_into().unwrap(),
        header_size: header_data[12..16].try_into().unwrap(),
        header_crc32: header_data[16..20].try_into().unwrap(),
        reserved: header_data[20..24].try_into().unwrap(),
        current_lba: header_data[24..32].try_into().unwrap(),
        backup_lba: header_data[32..40].try_into().unwrap(),
        first_usable_lba: header_data[40..48].try_into().unwrap(),
        last_usable_lba: header_data[48..56].try_into().unwrap(),
        disk_guid: header_data[56..72].try_into().unwrap(),
        partition_entry_lba: header_data[72..80].try_into().unwrap(),
        number_of_partition_entries: header_data[80..84].try_into().unwrap(),
        size_of_partition_entry: header_data[84..88].try_into().unwrap(),
        partition_entry_array_crc32: header_data[88..92].try_into().unwrap(),
    };

    let partition_entry_lba = u64::from_le_bytes(header.partition_entry_lba);
    let number_of_partition_entries = u32::from_le_bytes(header.number_of_partition_entries);
    let size_of_partition_entry = u32::from_le_bytes(header.size_of_partition_entry);

    info!(
        "GPT Header: {} entries at LBA {}",
        number_of_partition_entries, partition_entry_lba
    );

    // Read partition entries
    let partition_entry_offset = partition_entry_lba * 512;
    disk.set_position(partition_entry_offset);

    debug!("Partition entry size: {} bytes", size_of_partition_entry);
    debug!("Partition entry offset: {} bytes", partition_entry_offset);

    for i in 0..number_of_partition_entries {
        // Ensure we're at the correct position for this partition entry
        let current_entry_offset =
            partition_entry_offset + (i as u64 * size_of_partition_entry as u64);
        disk.set_position(current_entry_offset);

        let mut entry_data = vec![0u8; size_of_partition_entry as usize];
        if read_exact(disk, &mut entry_data).is_err() {
            warn!("Failed to read partition entry {}", i);
            continue;
        }

        let entry = if size_of_partition_entry >= 128 {
            // Safely parse the partition entry
            let partition_type_guid: [u8; 16] = entry_data[0..16].try_into().unwrap();
            let unique_partition_guid: [u8; 16] = entry_data[16..32].try_into().unwrap();
            let starting_lba: [u8; 8] = entry_data[32..40].try_into().unwrap();
            let ending_lba: [u8; 8] = entry_data[40..48].try_into().unwrap();
            let attributes: [u8; 8] = entry_data[48..56].try_into().unwrap();

            // Read partition name as UTF-16LE
            let mut partition_name = [0u16; 36];
            for j in 0..36 {
                let offset = 56 + j * 2;
                if offset + 1 < entry_data.len() {
                    partition_name[j] =
                        u16::from_le_bytes([entry_data[offset], entry_data[offset + 1]]);
                }
            }

            GptPartitionEntry {
                partition_type_guid,
                unique_partition_guid,
                starting_lba,
                ending_lba,
                attributes,
                partition_name,
            }
        } else {
            continue;
        };

        // Check if partition is in use (all zeros means unused)
        if entry.partition_type_guid.iter().all(|&b| b == 0) {
            continue;
        }

        let starting_lba = u64::from_le_bytes(entry.starting_lba);
        let ending_lba = u64::from_le_bytes(entry.ending_lba);
        let size_bytes = (ending_lba - starting_lba + 1) * 512;

        // Convert partition name from UTF-16LE to UTF-8
        let name_str = {
            // First, copy the partition name to a local array to avoid packed field reference
            let mut name_utf16 = [0u16; 36];
            for j in 0..36 {
                name_utf16[j] = entry.partition_name[j];
            }

            // Find the null terminator
            let mut name_len = 36;
            for j in 0..36 {
                if name_utf16[j] == 0 {
                    name_len = j;
                    break;
                }
            }

            // Convert only the valid portion
            let name_slice = &name_utf16[..name_len];
            let name_str = String::from_utf16_lossy(name_slice);
            debug!(
                "Partition {}: UTF-16LE name (len={}): {:?}, UTF-8 name: '{}'",
                i, name_len, name_slice, name_str
            );
            name_str
        };

        if name_str.is_empty() {
            continue;
        }

        // Detect filesystem type and read UUID in one go
        let (filesystem_type, filesystem_uuid) = {
            let fs_type = detect_filesystem_type(disk, starting_lba);
            let uuid = if let Some(ref fs) = fs_type {
                read_filesystem_uuid_simple(disk, starting_lba, fs)
            } else {
                None
            };
            (fs_type, uuid)
        };

        let partition = PartitionInfo {
            index: i as u32,
            name: name_str,
            partition_type_guid: entry.partition_type_guid,
            unique_partition_guid: entry.unique_partition_guid,
            filesystem_uuid,
            starting_lba,
            ending_lba,
            size_bytes,
            filesystem_type,
        };

        info!(
            "Found GPT partition {}: '{}' ({} bytes) with filesystem: {:?}",
            partition.index, partition.name, partition.size_bytes, partition.filesystem_type,
        );

        partitions.push(partition);
    }

    Ok(partitions)
}

/// Detect filesystem type on a partition
fn detect_filesystem_type(disk: &mut Disk, start_lba: u64) -> Option<FilesystemType> {
    let mut boot_sector = [0u8; 512];

    // Save current position
    let original_position = disk.position();

    // Set position to read from the specific LBA
    disk.set_position(start_lba * 512);

    if let Err(_) = read_exact(disk, &mut boot_sector) {
        warn!("Failed to read boot sector at LBA {}", start_lba);
        // Restore position
        disk.set_position(original_position);
        return None;
    }

    // Restore position
    disk.set_position(original_position);

    // Check for FAT filesystem
    if is_fat_filesystem(&boot_sector) {
        debug!("Detected FAT filesystem at LBA {}", start_lba);
        return Some(FilesystemType::Fat);
    }

    // Check for ext4 filesystem
    if is_ext4_filesystem(disk, start_lba) {
        debug!("Detected ext4 filesystem at LBA {}", start_lba);
        return Some(FilesystemType::Ext4);
    }

    debug!("Unknown filesystem type at LBA {}", start_lba);
    None
}

/// Read exactly the requested number of bytes
fn read_exact(disk: &mut Disk, mut buf: &mut [u8]) -> Result<(), ()> {
    while !buf.is_empty() {
        match disk.read_one(buf) {
            Ok(0) => break,
            Ok(n) => buf = &mut buf[n..],
            Err(_) => return Err(()),
        }
    }
    Ok(())
}

/// Check if the boot sector indicates a FAT filesystem
fn is_fat_filesystem(boot_sector: &[u8; 512]) -> bool {
    // Check for FAT12/FAT16/FAT32 signature at offset 0x36 (FAT) or 0x52 (FAT32)
    if boot_sector.len() >= 0x36 + 3 {
        let fat_sig = &boot_sector[0x36..0x36 + 3];
        if fat_sig == b"FAT" {
            return true;
        }
    }

    if boot_sector.len() >= 0x52 + 5 {
        let fat32_sig = &boot_sector[0x52..0x52 + 5];
        if fat32_sig == b"FAT32" {
            return true;
        }
    }

    false
}

/// Check if the partition contains an ext4 filesystem
fn is_ext4_filesystem(disk: &mut Disk, start_lba: u64) -> bool {
    // ext4 superblock is at offset 1024 (2 sectors) from the start of the partition
    let superblock_offset = start_lba * 512 + 1024;
    let mut superblock = [0u8; 2048]; // Increase buffer size to accommodate the magic number offset

    // Save current position
    let pos = disk.position();

    // Set position to read the superblock
    disk.set_position(superblock_offset);

    let result = if let Err(_) = read_exact(disk, &mut superblock) {
        warn!(
            "Failed to read ext4 superblock at offset {}",
            superblock_offset
        );
        false
    } else {
        // Check for ext4 magic number (0xEF53) at offset 1080 (0x438) in the superblock
        // But since we're reading from offset 1024, the magic number will be at index 56
        if superblock.len() >= 58 {
            let magic = u16::from_le_bytes([superblock[56], superblock[57]]);
            magic == 0xEF53
        } else {
            false
        }
    };

    // Restore position
    disk.set_position(pos);

    result
}

/// Create a filesystem instance for the given partition and filesystem type
pub fn create_filesystem_for_partition(
    disk: Disk,
    partition: &PartitionInfo,
) -> AxResult<Arc<dyn VfsOps>> {
    match partition.filesystem_type {
        Some(FilesystemType::Fat) => {
            info!("Creating FAT filesystem for partition '{}'", partition.name);
            // Create a partition wrapper
            let partition_wrapper =
                crate::dev::Partition::new(disk, partition.starting_lba, partition.ending_lba);
            let fs = crate::fs::fatfs::FatFileSystem::from_partition(partition_wrapper);
            Ok(Arc::new(fs))
        }
        Some(FilesystemType::Ext4) => {
            info!(
                "Creating ext4 filesystem for partition '{}'",
                partition.name
            );
            // Create a partition wrapper
            let partition_wrapper =
                crate::dev::Partition::new(disk, partition.starting_lba, partition.ending_lba);
            let fs = crate::fs::ext4fs::Ext4FileSystem::from_partition(partition_wrapper);
            Ok(Arc::new(fs))
        }
        Some(FilesystemType::Unknown) | None => {
            warn!("Unknown filesystem type for partition '{}'", partition.name);
            ax_err!(Unsupported, "Unknown filesystem type")
        }
    }
}

/// Read filesystem UUID directly from disk without mounting
/// This reads the UUID from the filesystem superblock
fn read_filesystem_uuid_simple(
    disk: &mut Disk,
    starting_lba: u64,
    filesystem_type: &FilesystemType,
) -> Option<String> {
    match filesystem_type {
        FilesystemType::Ext4 => read_ext4_uuid(disk, starting_lba),
        FilesystemType::Fat => read_fat32_uuid(disk, starting_lba),
        _ => None,
    }
}

/// Read UUID from ext4 filesystem superblock
fn read_ext4_uuid(disk: &mut Disk, starting_lba: u64) -> Option<String> {
    // Ext4 superblock is at offset 1024 bytes from the start of the partition
    let superblock_offset = starting_lba * 512 + 1024;

    // Set position to superblock
    disk.set_position(superblock_offset);

    // Read the superblock (ext4 superblock is 1024 bytes)
    let mut superblock_data = vec![0u8; 1024];
    let mut total_read = 0;

    // Read in chunks since read_one might not read all at once
    while total_read < 1024 {
        match disk.read_one(&mut superblock_data[total_read..]) {
            Ok(0) => break, // EOF
            Ok(n) => total_read += n,
            Err(_) => return None,
        }
    }

    // UUID is at offset 0x68 (104) in the superblock, 16 bytes long
    if superblock_data.len() >= 120 {
        let uuid_bytes = &superblock_data[104..120];

        // Convert UUID bytes to string format (8-4-4-4-12)
        // ext4 stores UUID as little-endian for the first 3 fields and big-endian for the last 2 fields
        let uuid_str = format!(
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            uuid_bytes[0],
            uuid_bytes[1],
            uuid_bytes[2],
            uuid_bytes[3], // Little endian for first 4 bytes
            uuid_bytes[4],
            uuid_bytes[5], // Little endian for next 2 bytes
            uuid_bytes[6],
            uuid_bytes[7], // Little endian for next 2 bytes
            uuid_bytes[8],
            uuid_bytes[9], // Big endian for next 2 bytes
            uuid_bytes[10],
            uuid_bytes[11],
            uuid_bytes[12],
            uuid_bytes[13],
            uuid_bytes[14],
            uuid_bytes[15] // Big endian for last 6 bytes
        );

        Some(uuid_str)
    } else {
        None
    }
}

/// Read UUID from FAT32 filesystem
fn read_fat32_uuid(disk: &mut Disk, starting_lba: u64) -> Option<String> {
    // FAT32 boot sector is at the start of the partition
    let boot_sector_offset = starting_lba * 512;

    // Set position to boot sector
    disk.set_position(boot_sector_offset);

    // Read the boot sector (512 bytes)
    let mut boot_sector = vec![0u8; 512];
    let mut total_read = 0;

    // Read in chunks since read_one might not read all at once
    while total_read < 512 {
        match disk.read_one(&mut boot_sector[total_read..]) {
            Ok(0) => break, // EOF
            Ok(n) => total_read += n,
            Err(_) => return None,
        }
    }

    // FAT32 doesn't have a standard UUID like ext4, but it has a Volume ID
    // Volume ID is at offset 0x43 (67) in the boot sector, 4 bytes long
    if boot_sector.len() >= 71 {
        let volume_id_bytes = &boot_sector[67..71];

        // Format as 8-character hex string
        let volume_id_str = format!(
            "{:02x}{:02x}{:02x}{:02x}",
            volume_id_bytes[3],
            volume_id_bytes[2],
            volume_id_bytes[1],
            volume_id_bytes[0] // Little endian
        );

        Some(volume_id_str)
    } else {
        None
    }
}
