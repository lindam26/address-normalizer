// Splits a multi-line address entry (street line(s) followed by a
// "City, ST ZIP" line) into separate fields, on top of the per-line
// abbreviation normalize.rs already does.

use crate::normalize::{self, normalize_line};

#[derive(Debug, PartialEq, Eq, Default)]
pub struct AddressRecord {
    pub street: String,
    pub city: String,
    pub state: String,
    pub zip: String,
}

fn looks_like_zip(token: &str) -> bool {
    match token.len() {
        5 => token.chars().all(|c| c.is_ascii_digit()),
        10 => {
            let (prefix, suffix) = token.split_at(5);
            prefix.chars().all(|c| c.is_ascii_digit())
                && suffix.starts_with('-')
                && suffix[1..].chars().all(|c| c.is_ascii_digit())
        }
        _ => false,
    }
}

// Pulls city/state/zip off the tail of a normalized locality line like
// "SPRINGFIELD IL 62701". Matching from the tail means a multi-word city
// name ("SAN FRANCISCO") is left intact in whatever remains.
fn parse_locality(line: &str) -> (String, String, String) {
    let normalized = normalize_line(line);
    let mut tokens: Vec<&str> = normalized.split_whitespace().collect();

    let zip = match tokens.last() {
        Some(last) if looks_like_zip(last) => {
            let zip = last.to_string();
            tokens.pop();
            zip
        }
        _ => String::new(),
    };

    let state = match tokens.last() {
        Some(last) if normalize::is_state_abbr(last) => {
            let state = last.to_string();
            tokens.pop();
            state
        }
        _ => String::new(),
    };

    (tokens.join(" "), state, zip)
}

/// Parses one address entry made up of one or more consecutive non-blank
/// lines. The last line is checked for a trailing state and/or zip; if
/// one is found, everything before it is the street and whatever's left
/// on that last line is the city. If neither is found, the last line
/// doesn't look like a locality line at all, so the whole block is kept
/// as the street with no city/state/zip.
pub fn parse_block(lines: &[String]) -> AddressRecord {
    let lines: Vec<&str> = lines
        .iter()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect();

    if lines.is_empty() {
        return AddressRecord::default();
    }

    let last = lines[lines.len() - 1];
    let rest = &lines[..lines.len() - 1];
    let (mut city, state, zip) = parse_locality(last);

    let street_lines: &[&str] = if state.is_empty() && zip.is_empty() {
        city = String::new();
        &lines
    } else {
        rest
    };
    let street = street_lines
        .iter()
        .map(|line| normalize_line(line))
        .collect::<Vec<_>>()
        .join(" ");

    AddressRecord {
        street,
        city,
        state,
        zip,
    }
}

/// Renders a parsed record back into a single comparable line: "STREET,
/// CITY STATE ZIP". Falls back to just whichever half is present when
/// the other one is missing.
pub fn format_record(record: &AddressRecord) -> String {
    let mut locality = record.city.clone();
    for part in [&record.state, &record.zip] {
        if !part.is_empty() {
            if !locality.is_empty() {
                locality.push(' ');
            }
            locality.push_str(part);
        }
    }

    if record.street.is_empty() {
        locality
    } else if locality.is_empty() {
        record.street.clone()
    } else {
        format!("{}, {}", record.street, locality)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(values: &[&str]) -> Vec<String> {
        values.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn single_line_with_no_locality_is_all_street() {
        let record = parse_block(&lines(&["123 Main Street"]));
        assert_eq!(record.street, "123 MAIN ST");
        assert_eq!(record.city, "");
        assert_eq!(record.state, "");
        assert_eq!(record.zip, "");
    }

    #[test]
    fn two_line_block_splits_street_and_locality() {
        let record = parse_block(&lines(&["123 Main St", "Springfield, IL 62701"]));
        assert_eq!(record.street, "123 MAIN ST");
        assert_eq!(record.city, "SPRINGFIELD");
        assert_eq!(record.state, "IL");
        assert_eq!(record.zip, "62701");
    }

    #[test]
    fn three_line_block_keeps_secondary_unit_line_in_street() {
        let record = parse_block(&lines(&["123 Main St", "Apt 4B", "Springfield, IL 62701"]));
        assert_eq!(record.street, "123 MAIN ST APT 4B");
        assert_eq!(record.city, "SPRINGFIELD");
    }

    #[test]
    fn spelled_out_state_and_missing_zip_still_parses() {
        let record = parse_block(&lines(&["456 Elm Ave", "Trenton, New Jersey"]));
        assert_eq!(record.street, "456 ELM AVE");
        assert_eq!(record.city, "TRENTON");
        assert_eq!(record.state, "NJ");
        assert_eq!(record.zip, "");
    }

    #[test]
    fn zip_plus_four_is_recognized() {
        let record = parse_block(&lines(&["123 Main St", "Springfield, IL 62701-1234"]));
        assert_eq!(record.zip, "62701-1234");
    }

    #[test]
    fn locality_only_block_has_no_street() {
        let record = parse_block(&lines(&["Springfield, Illinois"]));
        assert_eq!(record.street, "");
        assert_eq!(record.city, "SPRINGFIELD");
        assert_eq!(record.state, "IL");
    }

    #[test]
    fn empty_block_produces_empty_record() {
        let record = parse_block(&lines(&[]));
        assert_eq!(record, AddressRecord::default());
    }

    #[test]
    fn format_joins_street_and_locality() {
        let record = parse_block(&lines(&["123 Main St", "Springfield, IL 62701"]));
        assert_eq!(format_record(&record), "123 MAIN ST, SPRINGFIELD IL 62701");
    }

    #[test]
    fn format_falls_back_to_street_only() {
        let record = parse_block(&lines(&["123 Main Street"]));
        assert_eq!(format_record(&record), "123 MAIN ST");
    }

    #[test]
    fn format_falls_back_to_locality_only() {
        let record = parse_block(&lines(&["Springfield, Illinois"]));
        assert_eq!(format_record(&record), "SPRINGFIELD IL");
    }
}
