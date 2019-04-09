extern crate byteorder;

use crate::*;
use byteorder::{LittleEndian, WriteBytesExt};
use std::collections::HashMap;
use std::io::prelude::*;

enum AssemblyLineResult {
    Bytes(Vec<u8>),
    Label(String),
    Section(String),
    Relocation(Relocation),
}

pub struct Relocation {
    typ: RelocationType,
    label: String,
    location: u64,
}

fn register_offset(reg: &str) -> u8 {
    let offsets = vec!["eax", "ecx", "edx", "ebx", "esp", "ebp", "esi", "edi"];
    offsets.iter().position(|&r| r == reg).unwrap() as u8
}

fn to_uint<T: HexAndDecimalConvertable>(str: &str) -> T {
    let s = str.trim();
    if s.len() > 1 && &s[0..2] == "0x" {
        T::from_hex_str(s.trim_left_matches("0x")).unwrap()
    } else {
        T::parse_decimal(s).unwrap()
    }
}

trait HexAndDecimalConvertable: Sized {
    fn from_hex_str(s: &str) -> Result<Self, &'static str>;
    fn parse_decimal(s: &str) -> Result<Self, &'static str>;
}

impl HexAndDecimalConvertable for u8 {
    fn from_hex_str(s: &str) -> Result<Self, &'static str> {
        u8::from_str_radix(s, 16).map_err(|_| "from_str_radix failed :(")
    }
    fn parse_decimal(s: &str) -> Result<Self, &'static str> {
        s.parse().map_err(|_| "u8::parse failed")
    }
}

impl HexAndDecimalConvertable for u32 {
    fn from_hex_str(s: &str) -> Result<Self, &'static str> {
        u32::from_str_radix(s, 16).map_err(|_| "from_str_radix failed :(")
    }
    fn parse_decimal(s: &str) -> Result<Self, &'static str> {
        s.parse().map_err(|_| "u32::parse failed")
    }
}

//fn call(arguments: Vec<&str>, location: u64) -> Vec<AssemblyLineResult> {
//    let label = arguments[0];
//    if panic_on_missing_label {
//        let label_location = labels.get(label).expect("Label not defined");
//        let jump_target = (*label_location as i32 - (location as i32 + 5)) as u32;
//        let mut ret = vec![0xe8];
//        ret.write_u32::<LittleEndian>(jump_target).unwrap();
//        vec![AssemblyLineResult::Bytes(ret)]
//    } else {
//        vec![AssemblyLineResult::Bytes(vec![0xe8, 0, 0, 0, 0])]
//    }
//}
//
//fn jmp(opcode: u8, arguments: Vec<&str>, location: u64) -> Vec<AssemblyLineResult> {
//    let label = arguments[0];
//    if panic_on_missing_label {
//        let label_location = labels.get(label).expect("Label not defined");
//        let jump_target = (*label_location as i8 - (location as i8 + 2)) as u64;
//        vec![AssemblyLineResult::Bytes(vec![opcode, jump_target as u8])]
//    } else {
//        vec![AssemblyLineResult::Bytes(vec![opcode, 0])]
//    }
//}

fn assemble_line(line: &str, location: u64) -> Vec<AssemblyLineResult> {
    if line.trim().len() == 0 {
        return vec![];
    }

    let mut parts = line.trim().splitn(2, " ");
    let op = parts.next().unwrap().trim();

    if op.chars().last().unwrap() == ':' {
        vec![AssemblyLineResult::Label(
            op.trim_right_matches(":").to_string(),
        )]
    } else {
        let mut arguments: Vec<&str> = parts.next().unwrap_or("").split(",").collect();
        arguments = arguments.iter().map(|a| a.trim()).collect();
        match op {
            "section" => vec![AssemblyLineResult::Section(arguments[0].to_string())],
            "syscall" => vec![AssemblyLineResult::Bytes(vec![0xf, 0x5])],
            "ret" => vec![AssemblyLineResult::Bytes(vec![0xc3])],
            "mov" => {
                let target = arguments[0];
                let source = arguments[1];
                let opcode = 0xb8 + register_offset(target);
                let mut ret = vec![opcode];
                if source.chars().next().unwrap().is_digit(10) {
                    let value: u32 = to_uint(source);
                    ret.write_u32::<LittleEndian>(value).unwrap();
                    vec![AssemblyLineResult::Bytes(ret)]
                } else {
                    vec![
                        AssemblyLineResult::Bytes(ret),
                        AssemblyLineResult::Relocation(Relocation {
                            typ: RelocationType::U32,
                            label: source.to_string(),
                            location: location + 1,
                        }),
                    ]
                }
            }
            //"jmp" => jmp(0xeb, arguments, location),
            //"je" => jmp(0x74, arguments, location),
            //"jg" => jmp(0x7f, arguments, location),
            //"jl" => jmp(0x7c, arguments, location),
            //"jle" => jmp(0x7e, arguments, location),
            "cmp" => {
                let target = arguments[0];
                let value = to_uint::<u8>(arguments[1]);
                let modrm = 0xf8 + register_offset(target);
                vec![AssemblyLineResult::Bytes(vec![0x83, modrm, value])]
            }
            //"call" => call(arguments, location),
            "db" => {
                let mut ret = vec![];
                for arg in &arguments {
                    if arg.as_bytes().first() == Some(&b'"') {
                        ret.extend_from_slice(arg.trim_matches('"').as_bytes());
                    } else {
                        ret.push(to_uint(arg));
                    }
                }
                vec![AssemblyLineResult::Bytes(ret)]
            }
            _ => panic!("Not implemented"),
        }
    }
}

pub fn assemble(text: &str) -> AssemblyResult {
    // A label has a name, a section name, and a location relative to that section.
    let mut labels: HashMap<String, (String, u64)> = HashMap::new();

    let mut sections: Vec<AssemblySection> = vec![];
    let mut relocations: Vec<Relocation> = vec![];
    let mut location: u64 = 0;
    let mut i: i32 = -1;
    for line in text.lines() {
        for result in assemble_line(&line, location) {
            match result {
                AssemblyLineResult::Bytes(bytes) => {
                    sections[i as usize].content.write(&bytes).unwrap();
                    location += bytes.len() as u64;
                    ()
                }
                AssemblyLineResult::Label(name) => {
                    labels.insert(name, (sections[sections.len() - 1].name.clone(), location));
                    ()
                }
                AssemblyLineResult::Section(name) => {
                    sections.push(AssemblySection {
                        name: name,
                        content: vec![],
                    });
                    i += 1;
                    location = 0;
                }
                AssemblyLineResult::Relocation(relocation) => {
                    match relocation.typ {
                        RelocationType::U32 => {
                            sections[i as usize].content.write(&[0 as u8; 4]).unwrap();
                            location += 4 as u64;
                        }
                        RelocationType::U64 => {
                            sections[i as usize].content.write(&[0 as u8; 8]).unwrap();
                            location += 8 as u64;
                        }
                    }
                    relocations.push(relocation);
                }
            }
        }
    }

    // Resolve relocations.
    let mut resolved_relocations = vec![];
    for relocation in relocations {
        let (section, addend) = labels
            .get(&relocation.label)
            .expect("Could not resolve label");
        resolved_relocations.push(ResolvedRelocation {
            location: relocation.location,
            typ: relocation.typ,
            section: section.to_string(),
            addend: *addend,
        });
    }

    AssemblyResult {
        sections: sections,
        relocations: resolved_relocations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_assembly(line: &str, expected: Vec<u8>) {
        let result = assemble_line(line, 0).remove(0);
        let assembly = match result {
            AssemblyLineResult::Bytes(bytes) => bytes,
            _ => panic!("Unexpected AssemblyLineResult type"),
        };
        assert_eq!(assembly, expected);
    }

    #[test]
    fn conversion() {
        assert_eq!(to_uint::<u32>("0x42"), 66);
        assert_eq!(to_uint::<u32>("42"), 42);
        assert_eq!(to_uint::<u32>("0x0"), 0);
        assert_eq!(to_uint::<u8>("0x0"), 0);
    }

    #[test]
    fn syscall() {
        assert_assembly("syscall", vec![0xf, 0x5]);
    }

    #[test]
    fn ret() {
        assert_assembly("ret", vec![0xc3]);
    }

    #[test]
    fn mov() {
        assert_assembly("mov eax, 60", vec![0xb8, 0x3c, 0, 0, 0]);
        assert_assembly("mov ebx, 0x42", vec![0xbb, 0x42, 0, 0, 0]);
        assert_assembly("mov ebx, 0x12345678", vec![0xbb, 0x78, 0x56, 0x34, 0x12]);
    }

    #[test]
    fn mov_with_reference() {
        let result =
            assemble("section .text\nmov esi, message\nsection .rodata\nmessage:\ndb \"Hello\"");
        assert_eq!(result.sections[0].name, ".text");
        assert_eq!(result.sections[1].name, ".rodata");
        assert_eq!(result.sections[0].content, vec![0xb8 + 6, 0, 0, 0, 0]);
        assert_eq!(result.relocations[0].section, ".rodata");
        assert_eq!(result.relocations[0].location, 1);
    }

    //#[test]
    //fn jmp() {
    //    assert_assembly("loop:\njmp loop"), vec![0xeb, 0xfe]);
    //    assert_assembly("loop:\nje loop"), vec![0x74, 0xfe]);
    //    assert_eq!(
    //        assemble("forever:\njmp skip\njmp forever\nskip:"),
    //        vec![0xeb, 0x02, 0xeb, 0xfc]
    //    );
    //}

    //#[test]
    //fn call() {
    //    assert_eq!(
    //        assemble("call loop\nret\nloop:"),
    //        vec![0xe8, 1, 0, 0, 0, 0xc3]
    //    );
    //}

    #[test]
    fn cmp() {
        assert_assembly("cmp eax, 5", vec![0x83, 0xf8, 5]);
    }

    #[test]
    fn db() {
        assert_assembly("db 0x42", vec![0x42]);
        assert_assembly("db 42", vec![42]);
        assert_assembly("db \"*\"", vec![42]);
        assert_assembly("db \"*\", 0x42, 42", vec![42, 0x42, 42]);
        assert_assembly("db \"hello\"", vec![104, 101, 108, 108, 111]);
    }
}
