extern crate byteorder;
use byteorder::{LittleEndian, WriteBytesExt};

use std::fs::File;
use std::io::prelude::*;

fn register_offset(reg: &str) -> u8 {
    let offsets = vec!["eax", "ecx", "edx", "ebx", "esp", "ebp", "esi", "edi"];
    offsets.iter().position(|&r| r == reg).unwrap() as u8
}

fn to_u32(str: &str) -> u32 {
    let s = str.trim();
    if &s[0..2] == "0x" {
        u32::from_str_radix(s.trim_left_matches("0x"), 16).unwrap()
    } else {
        s.parse::<u32>().unwrap()
    }
}

fn to_opcode(line: &str) -> Vec<u8> {
    let mut parts = line.splitn(2, " ");
    let op = parts.next().unwrap();
    let arguments: Vec<&str> = parts.next().unwrap_or("").split(",").collect();
    match op {
        "syscall" => vec![0xf, 0x5],
        "mov" => {
            let target = arguments[0];
            let source = arguments[1];
            let opcode = 0xb8 + register_offset(target);
            let value = to_u32(source);
            let mut ret = vec![opcode];
            ret.write_u32::<LittleEndian>(value);
            return ret;
        }
        _ => vec![],
    }
}

pub fn assemble(text: &str) -> Vec<u8> {
    let mut bytes = vec![];
    for line in text.lines() {
        bytes.write(&to_opcode(&line));
    }
    bytes
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
        assert_eq!(to_opcode("syscall"), vec![0xf, 0x5]);
    }

    #[test]
    fn mov() {
        assert_eq!(to_opcode("mov eax, 60"), vec![0xb8, 0x3c, 0, 0, 0]);
        assert_eq!(to_opcode("mov ebx, 0x42"), vec![0xbb, 0x42, 0, 0, 0]);
        assert_eq!(
            to_opcode("mov ebx, 0x12345678"),
            vec![0xbb, 0x78, 0x56, 0x34, 0x12]
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
    //    assert_eq!(to_opcode("cmp eax, 0"), vec![0x48, 0x83, 0xf8, 0]);
    //}
}
