extern crate byteorder;
use byteorder::{LittleEndian, WriteBytesExt};

use std::collections::HashMap;
use std::io::prelude::*;

enum AssemblyResult {
    Bytes(Vec<u8>),
    Label(String),
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

fn assemble_line(
    line: &str,
    location: u8,
    labels: &HashMap<String, u8>,
    panic_on_missing_label: bool,
) -> AssemblyResult {
    if line.trim().len() == 0 {
        return AssemblyResult::Bytes(vec![]);
    }

    let mut parts = line.trim().splitn(2, " ");
    let op = parts.next().unwrap().trim();

    if op.chars().last().unwrap() == ':' {
        AssemblyResult::Label(op.trim_right_matches(":").to_string())
    } else {
        let mut arguments: Vec<&str> = parts.next().unwrap_or("").split(",").collect();
        arguments = arguments.iter().map(|a| a.trim()).collect();
        match op {
            "syscall" => AssemblyResult::Bytes(vec![0xf, 0x5]),
            "mov" => {
                let target = arguments[0];
                let source = arguments[1];
                let opcode = 0xb8 + register_offset(target);
                let value = to_u32(source);
                let mut ret = vec![opcode];
                ret.write_u32::<LittleEndian>(value).unwrap();
                AssemblyResult::Bytes(ret)
            }
            "jmp" => {
                let label = arguments[0];
                if panic_on_missing_label {
                    let label_location = labels.get(label).expect("Label not defined");
                    let jump_target = (*label_location as i8 - (location as i8 + 2)) as u8;
                    AssemblyResult::Bytes(vec![0xeb, jump_target])
                } else {
                    AssemblyResult::Bytes(vec![0xeb, 0])
                }
            }
            _ => panic!("Not implemented"),
        }
    }
}

pub fn assemble(text: &str) -> Vec<u8> {
    let mut result = vec![];
    let mut labels = HashMap::new();

    // First pass: Assemble instructions to bytes, but don't fill in the locations from the labels.
    let empty = HashMap::new();
    let mut location: u8 = 0;
    for line in text.lines() {
        match assemble_line(&line, location, &empty, false) {
            AssemblyResult::Bytes(bytes) => location += bytes.len() as u8,
            AssemblyResult::Label(name) => {
                labels.insert(name, location);
            }
        };
    }
    // Second pass: Now that we know where the labels point to, assemble again.
    location = 0;
    for line in text.lines() {
        match assemble_line(&line, location, &labels, true) {
            AssemblyResult::Bytes(bytes) => {
                result.write(&bytes).unwrap();
                location += bytes.len() as u8;
                ()
            }
            AssemblyResult::Label(_) => (),
        };
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversion() {
        assert_eq!(to_u32("0x42"), 66);
        assert_eq!(to_u32("42"), 42);
        assert_eq!(to_u32("0x0"), 0);
    }

    #[test]
    fn syscall() {
        assert_eq!(assemble("syscall"), vec![0xf, 0x5]);
    }

    #[test]
    fn mov() {
        assert_eq!(assemble("mov eax, 60"), vec![0xb8, 0x3c, 0, 0, 0]);
        assert_eq!(assemble("mov ebx, 0x42"), vec![0xbb, 0x42, 0, 0, 0]);
        assert_eq!(
            assemble("mov ebx, 0x12345678"),
            vec![0xbb, 0x78, 0x56, 0x34, 0x12]
        );
    }

    #[test]
    fn jmp() {
        assert_eq!(assemble("loop:\njmp loop"), vec![0xeb, 0xfe]);
        assert_eq!(
            assemble("forever:\njmp skip\njmp forever\nskip:"),
            vec![0xeb, 0x02, 0xeb, 0xfc]
        );
    }

    #[test]
    fn ass() {
        assert_eq!(
            assemble("mov eax, 60\nsyscall"),
            vec![0xb8, 0x3c, 0, 0, 0, 0xf, 0x5]
        );
    }

    //#[test]
    //fn cmp() {
    //    assert_eq!(assemble_line("cmp eax, 0"), vec![0x48, 0x83, 0xf8, 0]);
    //}
}
