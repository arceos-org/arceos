#![feature(exit_status_error)]

use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::BufWriter,
    process::Command,
};

use object::{
    Object, ObjectSection,
    build::elf::{Builder, SectionData},
    read::elf::ElfFile64,
    write::StreamingBuffer,
};

const DEBUG_SECTIONS: &[&[u8]] = &[
    b"debug_abbrev",
    b"debug_addr",
    b"debug_aranges",
    b"debug_info",
    b"debug_line",
    b"debug_line_str",
    b"debug_ranges",
    b"debug_rnglists",
    b"debug_str",
    b"debug_str_offsets",
];

fn main() {
    let input = std::env::args().nth(1).expect("No input file provided");
    let objcopy = std::env::var("OBJCOPY").unwrap_or("llvm-objcopy".to_owned());

    let mut sections: BTreeMap<&[u8], &[u8]> = BTreeMap::new();

    let data = fs::read(&input).expect("Failed to read file");
    let elf: ElfFile64 = ElfFile64::parse(data.as_slice()).expect("Failed to parse ELF file");

    for sec in elf.sections() {
        let Ok(name) = sec.name_bytes() else {
            continue;
        };
        if name[0] != b'.' || !DEBUG_SECTIONS.contains(&&name[1..]) {
            continue;
        }
        sections.insert(&name[1..], sec.data().expect("Failed to get section data"));
    }

    // `object` cannot remove sections correctly, so we have to use `objcopy`.
    Command::new(objcopy)
        .arg(&input)
        .arg("--strip-debug")
        .status()
        .expect("Failed to run objcopy")
        .exit_ok()
        .expect("objcopy failed");

    let data = fs::read(&input).expect("Failed to read file");
    let mut builder = Builder::read(data.as_slice()).expect("Failed to parse ELF file");

    for sec in builder.sections.iter_mut() {
        if let Some(data) = sections.remove(&*sec.name) {
            sec.name.to_mut().insert(0, b'.');
            sec.data = SectionData::Data(data.into());
        }
    }

    let writer = BufWriter::new(
        File::options()
            .write(true)
            .truncate(true)
            .open(input)
            .expect("Failed to open file for writing"),
    );
    builder
        .write(&mut StreamingBuffer::new(writer))
        .expect("Failed to write ELF");
}
