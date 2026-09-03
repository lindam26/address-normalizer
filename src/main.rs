mod normalize;

use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::process::ExitCode;

fn process<R: BufRead, W: Write>(reader: R, mut writer: W) -> io::Result<()> {
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        writeln!(writer, "{}", normalize::normalize_line(&line))?;
    }
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let stdout = io::stdout();
    let mut out = stdout.lock();

    if args.is_empty() {
        let stdin = io::stdin();
        if let Err(err) = process(stdin.lock(), &mut out) {
            eprintln!("addrnorm: error reading stdin: {}", err);
            return ExitCode::FAILURE;
        }
        return ExitCode::SUCCESS;
    }

    let mut had_error = false;
    for path in &args {
        match File::open(path) {
            Ok(file) => {
                if let Err(err) = process(BufReader::new(file), &mut out) {
                    eprintln!("addrnorm: error reading {}: {}", path, err);
                    had_error = true;
                }
            }
            Err(err) => {
                eprintln!("addrnorm: cannot open {}: {}", path, err);
                had_error = true;
            }
        }
    }

    if had_error {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
