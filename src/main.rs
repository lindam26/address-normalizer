mod normalize;
mod parse;

use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::process::ExitCode;

// Dedupe is keyed on the normalized output rather than the raw input line,
// since the whole point is to catch "123 North Main St" and "123 Main
// Street" as the same address after they've been abbreviated the same way.
fn emit_if_new<W: Write>(
    writer: &mut W,
    seen: &mut Option<HashSet<String>>,
    line: String,
) -> io::Result<()> {
    if let Some(seen) = seen {
        if !seen.insert(line.clone()) {
            return Ok(());
        }
    }
    writeln!(writer, "{}", line)
}

fn process_lines<R: BufRead, W: Write>(
    reader: R,
    writer: &mut W,
    seen: &mut Option<HashSet<String>>,
) -> io::Result<()> {
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        emit_if_new(writer, seen, normalize::normalize_line(&line))?;
    }
    Ok(())
}

// Blank lines separate one address entry from the next; each entry's
// lines are handed to parse::parse_block as a unit rather than
// normalized independently.
fn process_blocks<R: BufRead, W: Write>(
    reader: R,
    writer: &mut W,
    seen: &mut Option<HashSet<String>>,
) -> io::Result<()> {
    let mut block: Vec<String> = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            if !block.is_empty() {
                let record = parse::format_record(&parse::parse_block(&block));
                emit_if_new(writer, seen, record)?;
                block.clear();
            }
            continue;
        }
        block.push(line);
    }
    if !block.is_empty() {
        let record = parse::format_record(&parse::parse_block(&block));
        emit_if_new(writer, seen, record)?;
    }
    Ok(())
}

fn process<R: BufRead, W: Write>(
    reader: R,
    writer: &mut W,
    blocks_mode: bool,
    seen: &mut Option<HashSet<String>>,
) -> io::Result<()> {
    if blocks_mode {
        process_blocks(reader, writer, seen)
    } else {
        process_lines(reader, writer, seen)
    }
}

fn main() -> ExitCode {
    let mut blocks_mode = false;
    let mut dedupe = false;
    let mut paths: Vec<String> = Vec::new();
    for arg in env::args().skip(1) {
        if arg == "--blocks" {
            blocks_mode = true;
        } else if arg == "--dedupe" {
            dedupe = true;
        } else {
            paths.push(arg);
        }
    }

    let stdout = io::stdout();
    let mut out = stdout.lock();
    // A single seen-set spans every input source, so the same address
    // repeated across two files (or a file and stdin) still collapses.
    let mut seen = if dedupe { Some(HashSet::new()) } else { None };

    if paths.is_empty() {
        let stdin = io::stdin();
        if let Err(err) = process(stdin.lock(), &mut out, blocks_mode, &mut seen) {
            eprintln!("addrnorm: error reading stdin: {}", err);
            return ExitCode::FAILURE;
        }
        return ExitCode::SUCCESS;
    }

    let mut had_error = false;
    for path in &paths {
        match File::open(path) {
            Ok(file) => {
                if let Err(err) = process(BufReader::new(file), &mut out, blocks_mode, &mut seen) {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn run(input: &str, blocks_mode: bool, dedupe: bool) -> String {
        let mut out: Vec<u8> = Vec::new();
        let mut seen = if dedupe { Some(HashSet::new()) } else { None };
        process(input.as_bytes(), &mut out, blocks_mode, &mut seen).unwrap();
        String::from_utf8(out).unwrap()
    }

    #[test]
    fn dedupe_drops_repeated_normalized_lines() {
        let input = "123 North Main Street\n123 Main St\n456 Elm Ave\n";
        assert_eq!(run(input, false, true), "123 N MAIN ST\n456 ELM AVE\n");
    }

    #[test]
    fn without_dedupe_repeats_are_kept() {
        let input = "123 North Main Street\n123 Main St\n";
        assert_eq!(run(input, false, false), "123 N MAIN ST\n123 MAIN ST\n");
    }

    #[test]
    fn dedupe_works_on_formatted_blocks() {
        let input = "123 Main St\nSpringfield, IL 62701\n\n123 Main Street\nSpringfield, Illinois 62701\n";
        assert_eq!(
            run(input, true, true),
            "123 MAIN ST, SPRINGFIELD IL 62701\n"
        );
    }
}
