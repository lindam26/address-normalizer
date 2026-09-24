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

const SECONDARY_UNIT_DESIGNATORS: &[(&str, &str)] = &[
    ("APARTMENT", "APT"),
    ("BASEMENT", "BSMT"),
    ("BUILDING", "BLDG"),
    ("DEPARTMENT", "DEPT"),
    ("FLOOR", "FL"),
    ("FRONT", "FRNT"),
    ("HANGAR", "HNGR"),
    ("LOBBY", "LBBY"),
    ("LOWER", "LOWR"),
    ("OFFICE", "OFC"),
    ("PENTHOUSE", "PH"),
    ("PIER", "PIER"),
    ("REAR", "REAR"),
    ("ROOM", "RM"),
    ("SLIP", "SLIP"),
    ("SPACE", "SPC"),
    ("STOP", "STOP"),
    ("SUITE", "STE"),
    ("TRAILER", "TRLR"),
    ("UNIT", "UNIT"),
    ("UPPER", "UPPR"),
];

const STATES: &[(&str, &str)] = &[
    ("ALABAMA", "AL"),
    ("ALASKA", "AK"),
    ("ARIZONA", "AZ"),
    ("ARKANSAS", "AR"),
    ("CALIFORNIA", "CA"),
    ("COLORADO", "CO"),
    ("CONNECTICUT", "CT"),
    ("DELAWARE", "DE"),
    ("DISTRICT OF COLUMBIA", "DC"),
    ("FLORIDA", "FL"),
    ("GEORGIA", "GA"),
    ("HAWAII", "HI"),
    ("IDAHO", "ID"),
    ("ILLINOIS", "IL"),
    ("INDIANA", "IN"),
    ("IOWA", "IA"),
    ("KANSAS", "KS"),
    ("KENTUCKY", "KY"),
    ("LOUISIANA", "LA"),
    ("MAINE", "ME"),
    ("MARYLAND", "MD"),
    ("MASSACHUSETTS", "MA"),
    ("MICHIGAN", "MI"),
    ("MINNESOTA", "MN"),
    ("MISSISSIPPI", "MS"),
    ("MISSOURI", "MO"),
    ("MONTANA", "MT"),
    ("NEBRASKA", "NE"),
    ("NEVADA", "NV"),
    ("NEW HAMPSHIRE", "NH"),
    ("NEW JERSEY", "NJ"),
    ("NEW MEXICO", "NM"),
    ("NEW YORK", "NY"),
    ("NORTH CAROLINA", "NC"),
    ("NORTH DAKOTA", "ND"),
    ("OHIO", "OH"),
    ("OKLAHOMA", "OK"),
    ("OREGON", "OR"),
    ("PENNSYLVANIA", "PA"),
    ("RHODE ISLAND", "RI"),
    ("SOUTH CAROLINA", "SC"),
    ("SOUTH DAKOTA", "SD"),
    ("TENNESSEE", "TN"),
    ("TEXAS", "TX"),
    ("UTAH", "UT"),
    ("VERMONT", "VT"),
    ("VIRGINIA", "VA"),
    ("WASHINGTON", "WA"),
    ("WEST VIRGINIA", "WV"),
    ("WISCONSIN", "WI"),
    ("WYOMING", "WY"),
];

fn lookup(table: &[(&str, &str)], token: &str) -> Option<&'static str> {
    table
        .iter()
        .find(|(long, _)| *long == token)
        .map(|(_, short)| *short)
}

/// True if `token` is already a USPS state/territory abbreviation (e.g.
/// "NJ"). Used by the block parser to recognize a locality line's tail
/// without re-deriving the abbreviation list.
pub fn is_state_abbr(token: &str) -> bool {
    STATES.iter().any(|(_, abbr)| *abbr == token)
}

// State names are matched as a run of consecutive words rather than a
// single token, since entries like "NEW YORK" and "DISTRICT OF COLUMBIA"
// span more than one word. Longest match wins so "NEW YORK" isn't cut down
// to matching just "NEW" against nothing and falling through unabbreviated.
fn match_state(words: &[String], start: usize) -> Option<(&'static str, usize)> {
    let max_len = STATES
        .iter()
        .map(|(name, _)| name.split(' ').count())
        .max()
        .unwrap_or(1);
    for len in (1..=max_len).rev() {
        if start + len > words.len() {
            continue;
        }
        let candidate = words[start..start + len].join(" ");
        if let Some(abbr) = lookup(STATES, &candidate) {
            return Some((abbr, len));
        }
    }
    None
}

/// Normalizes a single address line: uppercases, collapses whitespace, and
/// swaps spelled-out street suffixes, directionals, secondary unit
/// designators, and state names for their USPS abbreviations so that "123
/// North Main Street Apartment 4, Trenton, New Jersey" and "123 N Main St
/// Apt 4, Trenton, NJ" compare equal downstream.
pub fn normalize_line(line: &str) -> String {
    // Trailing commas/periods are common ("Main Street,") but we don't want
    // them baked into the abbreviation lookup.
    let words: Vec<String> = line
        .split_whitespace()
        .map(|raw| raw.trim_end_matches([',', '.']).to_uppercase())
        .collect();

    let mut out: Vec<String> = Vec::with_capacity(words.len());
    let mut i = 0;
    while i < words.len() {
        if let Some((abbr, consumed)) = match_state(&words, i) {
            out.push(abbr.to_string());
            i += consumed;
            continue;
        }

        let word = &words[i];
        let abbr = lookup(&STREET_SUFFIXES, word)
            .or_else(|| lookup(&DIRECTIONALS, word))
            .or_else(|| lookup(&SECONDARY_UNIT_DESIGNATORS, word))
            .map(str::to_string)
            .unwrap_or_else(|| word.clone());
        out.push(abbr);
        i += 1;
    }
    out.join(" ")
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

    #[test]
    fn abbreviates_secondary_unit_designator() {
        assert_eq!(
            normalize_line("123 Main Street Apartment 4B"),
            "123 MAIN ST APT 4B"
        );
    }

    #[test]
    fn abbreviates_suite_and_unit() {
        assert_eq!(normalize_line("456 Elm Ave Suite 200"), "456 ELM AVE STE 200");
        assert_eq!(normalize_line("789 Oak Dr Unit 3"), "789 OAK DR UNIT 3");
    }

    #[test]
    fn abbreviates_single_word_state() {
        assert_eq!(
            normalize_line("123 Main St, Springfield, Illinois"),
            "123 MAIN ST SPRINGFIELD IL"
        );
    }

    #[test]
    fn abbreviates_two_word_state() {
        assert_eq!(
            normalize_line("100 Elm St, Trenton, New Jersey 08608"),
            "100 ELM ST TRENTON NJ 08608"
        );
    }

    #[test]
    fn abbreviates_three_word_state() {
        assert_eq!(
            normalize_line("450 Elm Ave, Georgetown, District of Columbia"),
            "450 ELM AVE GEORGETOWN DC"
        );
    }

    #[test]
    fn leaves_already_abbreviated_state_alone() {
        assert_eq!(
            normalize_line("100 Elm St, Trenton, NJ 08608"),
            "100 ELM ST TRENTON NJ 08608"
        );
    }
}
