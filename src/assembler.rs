extern crate byteorder;

use crate::*;
use byteorder::{LittleEndian, WriteBytesExt};
use std::collections::HashMap;
use std::io::prelude::*;

enum AssemblyLineResult {
    Bytes(Vec<u8>),
    Label(String),
    Section(String),
}

fn register_offset(reg: &str) -> u8 {
    let offsets = vec!["eax", "ecx", "edx", "ebx", "esp", "ebp", "esi", "edi"];
    offsets.iter().position(|&r| r == reg).unwrap() as u8
}

fn to_u32(str: &str) -> u32 {
    let s = str.trim();
    if s.len() > 1 && &s[0..2] == "0x" {
        u32::from_str_radix(s.trim_left_matches("0x"), 16).unwrap()
    } else {
        s.parse::<u32>().unwrap()
    }
}

fn call(
    arguments: Vec<&str>,
    location: u8,
    labels: &HashMap<String, u8>,
    panic_on_missing_label: bool,
) -> AssemblyLineResult {
    let label = arguments[0];
    if panic_on_missing_label {
        let label_location = labels.get(label).expect("Label not defined");
        let jump_target = (*label_location as i32 - (location as i32 + 5)) as u32;
        let mut ret = vec![0xe8];
        ret.write_u32::<LittleEndian>(jump_target).unwrap();
        AssemblyLineResult::Bytes(ret)
    } else {
        AssemblyLineResult::Bytes(vec![0xe8, 0, 0, 0, 0])
    }
}

fn jmp(
    opcode: u8,
    arguments: Vec<&str>,
    location: u8,
    labels: &HashMap<String, u8>,
    panic_on_missing_label: bool,
) -> AssemblyLineResult {
    let label = arguments[0];
    if panic_on_missing_label {
        let label_location = labels.get(label).expect("Label not defined");
        let jump_target = (*label_location as i8 - (location as i8 + 2)) as u8;
        AssemblyLineResult::Bytes(vec![opcode, jump_target])
    } else {
        AssemblyLineResult::Bytes(vec![opcode, 0])
    }
}

fn assemble_line(
    line: &str,
    location: u8,
    labels: &HashMap<String, u8>,
    panic_on_missing_label: bool,
) -> AssemblyLineResult {
    if line.trim().len() == 0 {
        return AssemblyLineResult::Bytes(vec![]);
    }

    let mut parts = line.trim().splitn(2, " ");
    let op = parts.next().unwrap().trim();

    if op.chars().last().unwrap() == ':' {
        AssemblyLineResult::Label(op.trim_right_matches(":").to_string())
    } else {
        let mut arguments: Vec<&str> = parts.next().unwrap_or("").split(",").collect();
        arguments = arguments.iter().map(|a| a.trim()).collect();
        match op {
            "section" => AssemblyLineResult::Section(arguments[0].to_string()),
            "syscall" => AssemblyLineResult::Bytes(vec![0xf, 0x5]),
            "ret" => AssemblyLineResult::Bytes(vec![0xc3]),
            "mov" => {
                let target = arguments[0];
                let source = arguments[1];
                let opcode = 0xb8 + register_offset(target);
                let mut ret = vec![opcode];
                if source.chars().next().unwrap().is_digit(10) {
                    let value = to_u32(source);
                    ret.write_u32::<LittleEndian>(value).unwrap();
                } else {
                    let label_location = labels.get(source).expect("Label not defined");
                    ret.write_u32::<LittleEndian>(*label_location as u32)
                        .unwrap();
                }
                AssemblyLineResult::Bytes(ret)
            }
            "jmp" => jmp(0xeb, arguments, location, labels, panic_on_missing_label),
            "je" => jmp(0x74, arguments, location, labels, panic_on_missing_label),
            "jg" => jmp(0x7f, arguments, location, labels, panic_on_missing_label),
            "jl" => jmp(0x7c, arguments, location, labels, panic_on_missing_label),
            "jle" => jmp(0x7e, arguments, location, labels, panic_on_missing_label),
            "cmp" => {
                let target = arguments[0];
                let value = to_u32(arguments[1]) as u8;
                let modrm = 0xf8 + register_offset(target);
                AssemblyLineResult::Bytes(vec![0x83, modrm, value])
            }
            "call" => call(arguments, location, labels, panic_on_missing_label),
            "db" => {
                let mut ret = vec![];
                for arg in &arguments {
                    if arg.chars().next().unwrap() == '"' {
                        for char in arg.trim_left_matches("\"").trim_right_matches("\"").chars() {
                            ret.push(char as u8);
                        }
                    } else {
                        ret.push(to_u32(arg) as u8);
                    }
                }
                AssemblyLineResult::Bytes(ret)
            }
            _ => panic!("Not implemented"),
        }
    }
}

pub fn assemble(text: &str) -> AssemblyResult {
    let mut labels = HashMap::new();

    // First pass: Assemble instructions to bytes, but don't fill in the locations from the labels.
    let empty = HashMap::new();
    let mut location: u8 = 0;
    for line in text.lines() {
        match assemble_line(&line, location, &empty, false) {
            AssemblyLineResult::Bytes(bytes) => location += bytes.len() as u8,
            AssemblyLineResult::Label(name) => {
                labels.insert(name, location);
            }
            AssemblyLineResult::Section(_) => {}
        };
    }

    // Second pass: Now that we know where the labels point to, assemble again.
    let mut sections: Vec<AssemblySection> = vec![];
    let mut location = 0;
    let mut i: i32 = -1;
    for line in text.lines() {
        match assemble_line(&line, location, &labels, true) {
            AssemblyLineResult::Bytes(bytes) => {
                sections[i as usize].content.write(&bytes).unwrap();
                location += bytes.len() as u8;
                ()
            }
            AssemblyLineResult::Label(_) => (),
            AssemblyLineResult::Section(name) => {
                sections.push(AssemblySection {
                    name: name,
                    content: vec![],
                });
                i += 1;
            }
        };
    }

    AssemblyResult { sections: sections }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_assembly(line: &str, expected: Vec<u8>) {
        let assembly = match assemble_line(line, 0, &HashMap::new(), false) {
            AssemblyLineResult::Bytes(bytes) => bytes,
            _ => panic!("Unexpected AssemblyLineResult type"),
        };
        assert_eq!(assembly, expected);
    }

    #[test]
    fn conversion() {
        assert_eq!(to_u32("0x42"), 66);
        assert_eq!(to_u32("42"), 42);
        assert_eq!(to_u32("0x0"), 0);
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
