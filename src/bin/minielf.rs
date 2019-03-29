extern crate byteorder;
use byteorder::{LittleEndian, WriteBytesExt};

use std::fs::File;
use std::io::prelude::*;

struct Segment {
    typ: u32,
    address: u64,
    size: u64,
    flags: u32,
    code: Vec<u8>,
}

fn main() -> std::io::Result<()> {
    let start_address = 0x6000000;
    let header_size = 4 + 4 + 8 + 8 * 2 + 2 * 4 + 3 * 8;
    let pht_entry_size = 2 * 4 + 6 * 8;

    let quit = vec![0xb8, 0x3c, 0, 0, 0, 0xbf, 0x2a, 0, 0, 0, 0xf, 0x5];

    let main_segment = Segment {
        typ: 1,
        address: start_address,
        size: header_size + pht_entry_size + quit.len() as u64,
        flags: 5,
        code: quit,
    };

    let segments = vec![main_segment];

    let mut buffer = File::create("out")?;

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

    // Instruction set architecture. x86 is 3.
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
    buffer.write_u64::<LittleEndian>(0)?;

    // "flags"
    buffer.write_u32::<LittleEndian>(0)?;

    // Size of the header.
    buffer.write_u16::<LittleEndian>(header_size as u16)?;

    // Size of a program header table entry.
    buffer.write_u16::<LittleEndian>(pht_entry_size as u16)?;

    // Number of entries in the program header table.
    buffer.write_u16::<LittleEndian>(segments.len() as u16)?;

    // Size of a section header table entry.
    buffer.write_u16::<LittleEndian>(0)?;

    // Number of entries in the section header table.
    buffer.write_u16::<LittleEndian>(0)?;

    // Index of section header table entry that contains section names.
    buffer.write_u16::<LittleEndian>(0)?;

    // Beginning of program header table.

    for segment in &segments {
        // Type of the segment. Loadable segment is 1.
        buffer.write_u32::<LittleEndian>(segment.typ)?;

        // Segment-dependent flags.
        buffer.write_u32::<LittleEndian>(segment.flags)?;

        // Offset.
        buffer.write_u64::<LittleEndian>(0)?;

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

    // Code.
    for segment in &segments {
        buffer.write(&segment.code)?;
    }

    Ok(())
}
