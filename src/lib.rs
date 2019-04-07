pub mod assembler;
pub mod elf;

pub struct AssemblySection {
    name: String,
    content: Vec<u8>,
}

pub struct AssemblyResult {
    sections: Vec<AssemblySection>,
}
