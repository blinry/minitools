use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let assembly = fs::read_to_string(&args[1])?;

    let result = minitools::assembler::assemble(&assembly);
    let binary = minitools::elf::create_binary(result)?;

    let filename = format!(
        "{}.o",
        Path::new(&args[1]).file_stem().unwrap().to_str().unwrap()
    );
    let mut buffer = File::create(&filename)?;

    buffer.write_all(&binary)?;
    Ok(())
}
