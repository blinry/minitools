extern crate byteorder;
use byteorder::{LittleEndian, WriteBytesExt};

use std::io::prelude::*;

struct Segment {
    typ: u32,
    flags: u32,
    offset: u64,
    address: u64,
    size: u64,
}

struct Section {
    name: String,
    typ: u32,
    flags: u64,
    contents: Vec<u8>,
}

fn section_names_bytes(sections: Vec<&Section>) -> Vec<u8> {
    let section_names: Vec<String> = sections.iter().map(|s| format!("{}\0", s.name)).collect();
    let mut ret = section_names.join("").to_string().into_bytes();
    ret.insert(0, 0);
    ret
}

pub fn create_binary(instructions: Vec<u8>) -> std::io::Result<Vec<u8>> {
    let start_address = 0x6000000;
    let header_size = 4 + 4 + 8 + 8 * 2 + 2 * 4 + 3 * 8;
    let pht_entry_size = 2 * 4 + 6 * 8;
    let sht_entry_size = 4 * 4 + 6 * 8;

    let code_section = Section {
        name: ".text".to_string(),
        typ: 1,
        flags: 6,
        contents: instructions,
    };

    let mut section_names_section = Section {
        name: ".shrtrtab".to_string(),
        typ: 3,
        flags: 0,
        contents: vec![],
    };

    let section_names_bytes = section_names_bytes(vec![&code_section, &section_names_section]);
    section_names_section.contents = section_names_bytes;
    let sections = vec![code_section, section_names_section];

    let content_sizes: Vec<u64> = sections.iter().map(|s| s.contents.len() as u64).collect();
    let content_size: u64 = content_sizes.iter().sum();

    let main_segment = Segment {
        typ: 1,
        flags: 5,
        offset: 0,
        address: start_address,
        size: header_size + pht_entry_size + content_size as u64,
    };

    let segments = vec![main_segment];

    let mut buffer = vec![];

    // Magic number: 0x7F plus "ELF".
    buffer.write(&[0x7f, 'E' as u8, 'L' as u8, 'F' as u8])?;

    // 32-bit format (1) or 64-bit format (2).
    buffer.write(&[2])?;

    // Little endian (1) or big endian (2).
    buffer.write(&[1])?;

    // ELF version. Original and curent version is 1.
    buffer.write(&[1])?;

    // Target OS ABI. Linux is 3, but it's often set to 0, regardless of platform.
    buffer.write(&[0])?;

    // Padding.
    buffer.write(&[0; 8])?;

    // Starting here, endianess goes into effect!

    // Object type: Executable is 2.
    buffer.write_u16::<LittleEndian>(2)?;

    // Instruction set architecture. x86 is 3, AMD64 is 62.
    buffer.write_u16::<LittleEndian>(62)?;

    // Always set to 1?
    buffer.write_u32::<LittleEndian>(1)?;

    // Address of the entry point.
    buffer.write_u64::<LittleEndian>(
        start_address + header_size + pht_entry_size * (segments.len() as u64),
    )?;

    // Start of the program header table.
    buffer.write_u64::<LittleEndian>(header_size)?;

    // Start of the section header table.
    buffer.write_u64::<LittleEndian>(
        header_size + pht_entry_size * (segments.len() as u64) + content_size,
    )?;

    // "flags"
    buffer.write_u32::<LittleEndian>(0)?;

    // Size of the header.
    buffer.write_u16::<LittleEndian>(header_size as u16)?;

    // Size of a program header table entry.
    buffer.write_u16::<LittleEndian>(pht_entry_size as u16)?;

    // Number of entries in the program header table.
    buffer.write_u16::<LittleEndian>(segments.len() as u16)?;

    // Size of a section header table entry.
    buffer.write_u16::<LittleEndian>(sht_entry_size as u16)?;

    // Number of entries in the section header table.
    buffer.write_u16::<LittleEndian>((sections.len() + 1) as u16)?;

    // Index of section header table entry that contains section names.
    buffer.write_u16::<LittleEndian>(sections.len() as u16)?;

    // Beginning of program header table.

    for segment in &segments {
        // Type of the segment. Loadable segment is 1.
        buffer.write_u32::<LittleEndian>(segment.typ)?;

        // Segment-dependent flags.
        buffer.write_u32::<LittleEndian>(segment.flags)?;

        // Offset.
        buffer.write_u64::<LittleEndian>(segment.offset)?;

        // Virtual address of the segment in memory.
        buffer.write_u64::<LittleEndian>(segment.address)?;

        // Physical address of the segment in memory.
        buffer.write_u64::<LittleEndian>(segment.address)?;

        // Size of the segment in the file image.
        buffer.write_u64::<LittleEndian>(segment.size)?;

        // Size of the segment in memory.
        buffer.write_u64::<LittleEndian>(segment.size)?;

        // Alignment.
        buffer.write_u64::<LittleEndian>(0)?;
    }

    // Contents.
    for section in &sections {
        buffer.write(&section.contents)?;
    }

    // Beginning of section header table.
    let mut name_offset = 1;
    let mut offset = header_size + pht_entry_size * (segments.len() as u64);

    buffer.write(&[0 as u8; 64]);

    for section in &sections {
        buffer.write_u32::<LittleEndian>(name_offset)?;
        name_offset += (section.name.len() + 1) as u32;
        buffer.write_u32::<LittleEndian>(section.typ)?;
        buffer.write_u64::<LittleEndian>(section.flags)?;
        buffer.write_u64::<LittleEndian>(start_address + offset)?;
        buffer.write_u64::<LittleEndian>(offset)?;
        offset += section.contents.len() as u64;
        buffer.write_u64::<LittleEndian>(section.contents.len() as u64)?;
        buffer.write_u32::<LittleEndian>(0)?;
        buffer.write_u32::<LittleEndian>(0)?;
        buffer.write_u64::<LittleEndian>(0)?;
        buffer.write_u64::<LittleEndian>(0)?;
    }

    Ok(buffer)
}
