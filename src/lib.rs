pub mod assembler;
pub mod elf;

pub struct AssemblySection {
    name: String,
    content: Vec<u8>,
}

#[derive(Copy, Clone)]
pub enum RelocationType {
    // see http://refspecs.linuxbase.org/elf/x86_64-abi-0.98.pdf
    U32 = 10,
    U64 = 1,
}

pub struct ResolvedRelocation {
    location: u64,
    typ: RelocationType,
    section: String,
    addend: u64,
}

pub struct AssemblyResult {
    sections: Vec<AssemblySection>,
    relocations: Vec<ResolvedRelocation>,
}
