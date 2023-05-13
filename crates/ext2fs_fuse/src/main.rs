#![allow(unused)]
use clap::{App, Arg};
use ext2fs::{BlockDevice, Ext2FileSystem, BLOCK_SIZE, BLOCKS_PER_GRP, EXT2_S_IFDIR, EXT2_S_IFREG,
            TimeProvider, ZeroTimeProvider};
use std::fs::{read_dir, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use log::*;

const NUM_BLOCKS: usize = BLOCKS_PER_GRP;

struct BlockFile {
    file: Mutex<File>,
    num_blocks: usize
}

impl BlockDevice for BlockFile {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        let mut file = self.file.lock().unwrap();
        file.seek(SeekFrom::Start((block_id * BLOCK_SIZE) as u64))
            .expect("Error when seeking!");
        assert_eq!(file.read(buf).unwrap(), BLOCK_SIZE, "Not a complete block!");
    }

    fn write_block(&self, block_id: usize, buf: &[u8]) {
        let mut file = self.file.lock().unwrap();
        file.seek(SeekFrom::Start((block_id * BLOCK_SIZE) as u64))
            .expect("Error when seeking!");
        assert_eq!(file.write(buf).unwrap(), BLOCK_SIZE, "Not a complete block!");
    }

    fn block_num(&self) -> usize {
        self.num_blocks
    }

    fn block_size(&self) -> usize {
        BLOCK_SIZE
    }
}

impl BlockFile {
    pub fn new(f: File, num_blocks: usize) -> Self {
        f.set_len((BLOCK_SIZE * num_blocks) as u64);
        Self { file: Mutex::new(f), num_blocks }
    }
}

fn main() {
    env_logger::init();
    efs_test();
}

fn efs_test() -> std::io::Result<()> {
    let block_file = Arc::new(BlockFile::new(
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("target/fs.img")?, NUM_BLOCKS 
    ));
    Ext2FileSystem::create(block_file.clone(), Arc::new(ZeroTimeProvider));
    let efs = Ext2FileSystem::open(
        block_file.clone(), 
        Arc::new(ZeroTimeProvider)
    );

    let root_inode = Ext2FileSystem::root_inode(&efs);
    let filea = root_inode.create("filea", EXT2_S_IFREG).unwrap();
    let fileb = root_inode.create("fileb", EXT2_S_IFREG).unwrap();
    let dirc = root_inode.create("dirc", EXT2_S_IFDIR).unwrap();
    let filed = dirc.create("filed", EXT2_S_IFREG).unwrap();
    let dire = dirc.create("dire", EXT2_S_IFDIR).unwrap();
    dire.create("filef", EXT2_S_IFREG);
    dire.create("dirg", EXT2_S_IFDIR).unwrap();
    dirc.link("filealink", filea.inode_id().unwrap());
    let filealink = dirc.find("filealink").unwrap();

    println!("After initialize");
    println!("Under root:");
    for name in root_inode.ls().unwrap() {
        println!("{}", name);
    }
    println!("Under dirc:");
    for name in dirc.ls().unwrap() {
        println!("{}", name);
    }
    println!("Under dire:");
    for name in dire.ls().unwrap() {
        println!("{}", name);
    }
    let greet_str = "Hello, world!";
    filea.write_at(0, greet_str.as_bytes());
    fileb.write_at(0, greet_str.as_bytes());

    // basic read and write
    let mut buffer = [0u8; 233];
    let len = filea.read_at(0, &mut buffer).unwrap();
    assert_eq!(greet_str, core::str::from_utf8(&buffer[..len]).unwrap());

    // ftruncate file
    assert!(fileb.ftruncate(4096).unwrap());
    assert!(fileb.ftruncate(4).unwrap());
    let lenb = fileb.read_at(0, &mut buffer).unwrap();
    println!("fileb content after truncate:");
    println!("{}", core::str::from_utf8(&buffer[..lenb]).unwrap());

    // write from another place
    filealink.append(greet_str.as_bytes());
    let lena = filea.read_at(0, &mut buffer).unwrap();
    println!("filea content after write from filealink:");
    println!("{}", core::str::from_utf8(&buffer[..lena]).unwrap());

    // rm file
    assert!(root_inode.rm_file("fileb").unwrap());
    println!("After remove fileb");
    println!("Under root:");
    for name in root_inode.ls().unwrap() {
        println!("{}", name);
    }

    // invalid
    assert!(fileb.disk_inode().is_none());

    // rm empty dir
    assert!(dire.rm_dir("dirg", false).unwrap());

    // rm non-empty dir FAIL
    assert!(!dirc.rm_dir("dire", false).unwrap());

    // rm non-empty dir recursively SUCCESS
    assert!(dirc.rm_dir("dire", true).unwrap());

    println!("After remove dire recursively, dirc:");
    for name in dirc.ls().unwrap() {
        println!("{}", name);
    }

    // link count
    let disk_inode_a_before = filealink.disk_inode().unwrap();
    assert_eq!(disk_inode_a_before.i_links_count, 2);
    assert!(dirc.rm_file("filealink").unwrap());
    let disk_inode_a_after = filealink.disk_inode().unwrap();
    assert_eq!(disk_inode_a_after.i_links_count, 1);

    assert!(root_inode.rm_dir("dirc", true).unwrap());

    let mut random_str_test = |len: usize| {
        filea.ftruncate(0);
        assert_eq!(filea.read_at(0, &mut buffer).unwrap(), 0,);
        let mut str = String::new();
        use rand;
        // random digit
        for _ in 0..len {
            str.push(char::from('0' as u8 + rand::random::<u8>() % 10));
        }
        filea.write_at(0, str.as_bytes());
        let mut read_buffer = [0u8; 127];
        let mut offset = 0usize;
        let mut read_str = String::new();
        loop {
            let len = filea.read_at(offset, &mut read_buffer).unwrap();
            if len == 0 {
                break;
            }
            offset += len;
            read_str.push_str(core::str::from_utf8(&read_buffer[..len]).unwrap());
        }
        assert_eq!(str, read_str);
    };

    random_str_test(4 * BLOCK_SIZE);
    random_str_test(8 * BLOCK_SIZE + BLOCK_SIZE / 2);
    random_str_test(100 * BLOCK_SIZE);
    random_str_test(70 * BLOCK_SIZE + BLOCK_SIZE / 7);
    random_str_test((12 + 128) * BLOCK_SIZE);
    random_str_test(400 * BLOCK_SIZE);
    random_str_test(1000 * BLOCK_SIZE);
    random_str_test(2000 * BLOCK_SIZE);

    Ok(())
}