use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let assembly = fs::read_to_string(&args[1])?;

    let bytes = minitools::assembler::assemble(&assembly);
    let binary = minitools::elf::create_binary(bytes)?;

    let filename = Path::new(&args[1]).file_stem().unwrap();
    let mut buffer = File::create(filename)?;

    let mut perms = fs::metadata(filename)?.permissions();
    perms.set_mode(perms.mode() | 0o700);
    fs::set_permissions(filename, perms)?;

    buffer.write(&binary);
    Ok(())
}
