use std::fs::File;
use std::io::prelude::*;

fn main() -> std::io::Result<()> {
    let mut buffer = File::create("out")?;

    // Magic number: 0x7F plus "ELF".
    buffer.write(&[0x7f, 'E' as u8, 'L' as u8, 'F' as u8])?;

    // 32-bit format (1) or 64-bit format (2).
    buffer.write(&[1])?;

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
    buffer.write(&[2, 0])?;

    // Instruction set architecture. x86 is 3.
    buffer.write(&[3, 0])?;

    // Always set to 1?
    buffer.write(&[1, 0, 0, 0])?;

    // Address of the entry point.
    buffer.write(&[0x60, 0, 0, 8])?; // correct?

    // Start of the program header table.
    buffer.write(&[0x40, 0, 0, 0])?; // correct?

    // Start of the section header table.
    buffer.write(&[0, 0, 0, 0])?;

    // "flags"
    buffer.write(&[0, 0, 0, 0])?;

    // Size of the header.
    buffer.write(&[0x34, 0])?;

    // Size of a program header table entry.
    buffer.write(&[32, 0])?;

    // Number of entries in the program header table.
    buffer.write(&[1, 0])?;

    // Size of a section header table entry.
    buffer.write(&[0, 0])?;

    // Number of entries in the section header table.
    buffer.write(&[0, 0])?;

    // Index of section header table entry that contains section names.
    buffer.write(&[0, 0])?;

    // Padding.
    buffer.write(&[0; 12])?;

    // Beginning of program header table.

    // Type of the segment. Loadable segment is 1.
    buffer.write(&[1, 0, 0, 0])?;

    // Offset.
    buffer.write(&[0, 0, 0, 0])?;

    // Virtual address of the segment in memory.
    buffer.write(&[0, 0, 0, 8])?;

    // Physical address of the segment in memory.
    buffer.write(&[0, 0, 0, 8])?;

    // Size of the segment in the file image.
    buffer.write(&[0x70, 0, 0, 0])?;

    // Size of the segment in memory.
    buffer.write(&[0x70, 0, 0, 0])?;

    // Segment-dependent flags.
    buffer.write(&[5, 0, 0, 0])?;

    // Alignment.
    buffer.write(&[0, 0, 0, 1])?;

    // Code.
    buffer.write(&[0xbb, 0x2a, 0, 0, 0, 0xb8, 1, 0, 0, 0, 0xcd, 0x80])?;

    Ok(())
}
