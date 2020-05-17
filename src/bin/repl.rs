extern crate byteorder;
extern crate rustyline;
extern crate tempfile;

use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

fn main() -> std::io::Result<()> {
    let mut rl = Editor::<()>::new();
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());

                let mut file = NamedTempFile::new()?;
                let file2 = NamedTempFile::new()?;

                writeln!(file, "BITS 64")?;
                writeln!(file, "{}", line)?;

                let output = Command::new("nasm")
                    .arg(file.path())
                    .arg("-o")
                    .arg(file2.path())
                    .output()?;
                print!("{}", std::str::from_utf8(&output.stdout).unwrap());
                print!("{}", std::str::from_utf8(&output.stderr).unwrap());

                if output.stderr.is_empty() {
                    let bytes = std::fs::read(file2.path())?;
                    for byte in &bytes {
                        print!("{:02x} ", byte);
                    }
                    //print!("| ");
                    //for byte in &bytes {
                    //    print!("{:08b} ", byte);
                    //}
                    println!();
                }
            }
            Err(ReadlineError::Interrupted) => {
                break;
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history("history.txt").unwrap();

    Ok(())
}
