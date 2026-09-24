mod normalize;
mod parse;

use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::process::ExitCode;

fn process_lines<R: BufRead, W: Write>(reader: R, writer: &mut W) -> io::Result<()> {
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        writeln!(writer, "{}", normalize::normalize_line(&line))?;
    }
    Ok(())
}

// Blank lines separate one address entry from the next; each entry's
// lines are handed to parse::parse_block as a unit rather than
// normalized independently.
fn process_blocks<R: BufRead, W: Write>(reader: R, writer: &mut W) -> io::Result<()> {
    let mut block: Vec<String> = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            if !block.is_empty() {
                writeln!(writer, "{}", parse::format_record(&parse::parse_block(&block)))?;
                block.clear();
            }
            continue;
        }
        block.push(line);
    }
    if !block.is_empty() {
        writeln!(writer, "{}", parse::format_record(&parse::parse_block(&block)))?;
    }
    Ok(())
}

fn process<R: BufRead, W: Write>(reader: R, writer: &mut W, blocks_mode: bool) -> io::Result<()> {
    if blocks_mode {
        process_blocks(reader, writer)
    } else {
        process_lines(reader, writer)
    }
}

fn main() -> ExitCode {
    let mut blocks_mode = false;
    let mut paths: Vec<String> = Vec::new();
    for arg in env::args().skip(1) {
        if arg == "--blocks" {
            blocks_mode = true;
        } else {
            paths.push(arg);
        }
    }

    let stdout = io::stdout();
    let mut out = stdout.lock();

    if paths.is_empty() {
        let stdin = io::stdin();
        if let Err(err) = process(stdin.lock(), &mut out, blocks_mode) {
            eprintln!("addrnorm: error reading stdin: {}", err);
            return ExitCode::FAILURE;
        }
        return ExitCode::SUCCESS;
    }

    let mut had_error = false;
    for path in &paths {
        match File::open(path) {
            Ok(file) => {
                if let Err(err) = process(BufReader::new(file), &mut out, blocks_mode) {
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
