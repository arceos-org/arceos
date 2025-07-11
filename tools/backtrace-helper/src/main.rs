use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::BufWriter,
};

use object::{
    build::elf::{Builder, SectionData},
    write::StreamingBuffer,
};

const DEBUG_SECTIONS: &[&str] = &[
    "debug_abbrev",
    "debug_addr",
    "debug_aranges",
    "debug_info",
    "debug_line",
    "debug_line_str",
    "debug_ranges",
    "debug_rnglists",
    "debug_str",
    "debug_str_offsets",
];

fn main() {
    let input = std::env::args().nth(1).expect("No input file provided");

    let data = fs::read(&input).expect("Failed to read file");
    let mut builder = Builder::read(data.as_slice()).expect("Failed to parse ELF");

    let mut sections: BTreeMap<&str, SectionData> = BTreeMap::new();

    for sec in builder.sections.iter_mut() {
        let Some(name) = DEBUG_SECTIONS
            .iter()
            .find(|&name| sec.name.as_slice() == format!(".{name}").as_bytes())
        else {
            continue;
        };
        sections.insert(name, sec.data.clone());
    }

    for sec in builder.sections.iter_mut() {
        let Some(name) = DEBUG_SECTIONS
            .iter()
            .find(|&name| sec.name.as_slice() == name.as_bytes())
        else {
            continue;
        };
        sec.name.to_mut().insert(0, b'.');
        if let Some(data) = sections.remove(name) {
            sec.data = data;
        }
    }

    builder.delete_orphans();

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
