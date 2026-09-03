// USPS Publication 28 abbreviation tables, trimmed to the entries people
// actually type by hand. Extend these as real-world input surfaces gaps.

const STREET_SUFFIXES: &[(&str, &str)] = &[
    ("STREET", "ST"),
    ("AVENUE", "AVE"),
    ("BOULEVARD", "BLVD"),
    ("DRIVE", "DR"),
    ("COURT", "CT"),
    ("LANE", "LN"),
    ("ROAD", "RD"),
    ("PLACE", "PL"),
    ("SQUARE", "SQ"),
    ("TERRACE", "TER"),
    ("CIRCLE", "CIR"),
    ("PARKWAY", "PKWY"),
    ("HIGHWAY", "HWY"),
    ("TRAIL", "TRL"),
    ("WAY", "WAY"),
    ("ALLEY", "ALY"),
    ("LOOP", "LOOP"),
    ("EXTENSION", "EXT"),
    ("JUNCTION", "JCT"),
    ("CROSSING", "XING"),
];

const DIRECTIONALS: &[(&str, &str)] = &[
    ("NORTH", "N"),
    ("SOUTH", "S"),
    ("EAST", "E"),
    ("WEST", "W"),
    ("NORTHEAST", "NE"),
    ("NORTHWEST", "NW"),
    ("SOUTHEAST", "SE"),
    ("SOUTHWEST", "SW"),
];

fn lookup(table: &[(&str, &str)], token: &str) -> Option<&'static str> {
    table
        .iter()
        .find(|(long, _)| *long == token)
        .map(|(_, short)| *short)
}

/// Normalizes a single address line: uppercases, collapses whitespace, and
/// swaps spelled-out street suffixes and directionals for their USPS
/// abbreviations so that "123 North Main Street" and "123 N Main St" compare
/// equal downstream.
pub fn normalize_line(line: &str) -> String {
    let words: Vec<String> = line
        .split_whitespace()
        .map(|raw| {
            // Trailing commas/periods are common ("Main Street,") but we
            // don't want them baked into the abbreviation lookup.
            let trimmed = raw.trim_end_matches([',', '.']);
            let upper = trimmed.to_uppercase();
            lookup(&STREET_SUFFIXES, &upper)
                .or_else(|| lookup(&DIRECTIONALS, &upper))
                .map(str::to_string)
                .unwrap_or(upper)
        })
        .collect();
    words.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abbreviates_suffix() {
        assert_eq!(normalize_line("123 Main Street"), "123 MAIN ST");
    }

    #[test]
    fn handles_directional_and_suffix_together() {
        assert_eq!(normalize_line("456 North Elm Avenue"), "456 N ELM AVE");
    }

    #[test]
    fn strips_trailing_punctuation_before_matching() {
        assert_eq!(normalize_line("123 Main Street,"), "123 MAIN ST");
    }

    #[test]
    fn collapses_repeated_whitespace() {
        assert_eq!(normalize_line("123   Main    Street"), "123 MAIN ST");
    }

    #[test]
    fn leaves_already_abbreviated_input_alone() {
        assert_eq!(normalize_line("123 Main St"), "123 MAIN ST");
    }
}
