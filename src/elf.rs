extern crate byteorder;
use crate::*;
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
    content: Vec<u8>,
    link: u32,
    entry_size: u64,
}

fn section_names_bytes(sections: &Vec<Section>) -> Vec<u8> {
    let section_names: Vec<String> = sections.iter().map(|s| format!("{}\0", s.name)).collect();
    let mut ret = section_names.join("").to_string().into_bytes();
    ret.insert(0, 0);
    ret
}

fn string_bytes(symbols: &Vec<Symbol>) -> Vec<u8> {
    let section_names: Vec<String> = symbols.iter().map(|s| format!("{}\0", s.name)).collect();
    let mut ret = section_names.join("").to_string().into_bytes();
    ret.insert(0, 0);
    ret
}

fn symbol_bytes(symbols: &Vec<Symbol>) -> Vec<u8> {
    let mut ret = vec![];
    let mut offset = 1;
    for symbol in symbols {
        ret.write_u32::<LittleEndian>(offset).unwrap();
        offset += symbol.name.len() as u32 + 1;
        ret.write(&[symbol.info]).unwrap();
        ret.write(&[symbol.other]).unwrap();
        ret.write_u16::<LittleEndian>(symbol.section).unwrap();
        ret.write_u64::<LittleEndian>(symbol.value).unwrap();
        ret.write_u64::<LittleEndian>(symbol.size).unwrap();
    }
    ret
}

struct Symbol {
    name: String,
    info: u8,
    other: u8,
    section: u16,
    value: u64,
    size: u64,
}

pub fn create_binary(assembly: AssemblyResult) -> std::io::Result<Vec<u8>> {
    let start_address = 0x6000000;
    let header_size = 4 + 4 + 8 + 8 * 2 + 2 * 4 + 3 * 8;
    let pht_entry_size = 2 * 4 + 6 * 8;
    let sht_entry_size = 4 * 4 + 6 * 8;

    let mut sections = vec![];

    for s in assembly.sections {
        let flags = if s.name == ".rodata" { 2 } else { 6 };
        let section = Section {
            name: s.name,
            typ: 1,
            flags: flags,
            content: s.content,
            link: 0,
            entry_size: 0,
        };
        sections.push(section);
    }

    let mut symbols = vec![];

    let test_symbol = Symbol {
        name: "test".to_string(),
        info: 0,
        other: 0,
        section: 1,
        value: 0x42,
        size: 0x23,
    };

    symbols.push(test_symbol);

    let string_table = Section {
        name: ".strtab".to_string(),
        typ: 3, // SHT_STRTAB
        flags: 0,
        content: string_bytes(&symbols),
        link: 0,
        entry_size: 0,
    };
    sections.push(string_table);

    let symbol_table = Section {
        name: ".symtab".to_string(),
        typ: 2, // SHT_SYMTAB
        flags: 0,
        content: symbol_bytes(&symbols),
        link: (sections.iter().position(|s| s.name == ".strtab").unwrap() + 1) as u32,
        entry_size: 24,
    };
    sections.push(symbol_table);

    let section_names_section = Section {
        name: ".shrtrtab".to_string(),
        typ: 3,
        flags: 0,
        content: vec![],
        link: 0,
        entry_size: 0,
    };

    sections.push(section_names_section);
    let i = sections.len() - 1;
    sections[i].content = section_names_bytes(&sections);

    let content_sizes: Vec<u64> = sections.iter().map(|s| s.content.len() as u64).collect();
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

    // content.
    for section in &sections {
        buffer.write(&section.content)?;
    }

    // Beginning of section header table.

    let mut name_offset = 1;
    let mut offset = header_size + pht_entry_size * (segments.len() as u64);

    // First entry is filled with zeroes by convention.
    buffer.write(&[0 as u8; 64]).unwrap();

    for section in &sections {
        // Offset of this section's name in the string table this section links to.
        buffer.write_u32::<LittleEndian>(name_offset)?;
        name_offset += (section.name.len() + 1) as u32;

        // Type. PROGBITS is 1, SYMTAB is 2, STRTAB is 3.
        buffer.write_u32::<LittleEndian>(section.typ)?;

        // Flags, to mark if this section is writable or executable.
        buffer.write_u64::<LittleEndian>(section.flags)?;

        // Address at which the first byte of this entry will be loaded.
        buffer.write_u64::<LittleEndian>(start_address + offset)?;

        // Offset from the beginning of the file of this section.
        buffer.write_u64::<LittleEndian>(offset)?;

        // Size of this section in bytes.
        offset += section.content.len() as u64;
        buffer.write_u64::<LittleEndian>(section.content.len() as u64)?;

        // Linked section. Interpretation depends on this section's type.
        buffer.write_u32::<LittleEndian>(section.link)?;

        // Extra information. Interpretation depends on this section's type.
        buffer.write_u32::<LittleEndian>(0)?;

        // Alignment constraint.
        buffer.write_u64::<LittleEndian>(0)?;

        // Size of one entry, if this section contains fixed-size entries.
        buffer.write_u64::<LittleEndian>(section.entry_size)?;
    }

    Ok(buffer)
}
