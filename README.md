# address-normalizer

Freeform address input never comes in one shape. A signup form, a CSV
export, and a support ticket will spell the same address three different
ways: "123 North Main Street", "123 N. Main St", "123 north main st". If
you're trying to deduplicate addresses or match them against a reference
list, that inconsistency is the whole problem.

`addrnorm` takes address lines and rewrites them to a consistent form:
uppercase, single spaces, spelled-out directionals, street suffixes,
secondary unit designators (Apartment, Suite, Unit, ...), and state names
collapsed to their USPS abbreviations. It doesn't validate that an address
exists or geocode anything - it just makes equivalent input look the same,
so string comparison and deduplication actually work.

## Usage

Read from a file, one address line per line:

```
addrnorm addresses.txt
```

Read from stdin, so it fits into a pipeline:

```
cat addresses.txt | addrnorm
grep -i "main" export.csv | addrnorm
```

Multiple files are processed in order, output goes to stdout:

```
addrnorm north.txt south.txt > combined.txt
```

Example:

```
$ printf '123 North Main Street\n456 west  elm avenue,\n789 Oak Dr Apartment 4B\nTrenton, New Jersey\n' | addrnorm
123 N MAIN ST
456 W ELM AVE
789 OAK DR APT 4B
TRENTON NJ
```

Blank lines are skipped. Each input line is treated as one address line.

## Multi-line blocks

Real address input is often one entry per several lines rather than one
per line - a street line (or two, if there's a suite/apartment line)
followed by a "City, ST ZIP" line, with a blank line before the next
entry. Pass `--blocks` to parse that shape into street/city/state/zip
fields instead of normalizing each line on its own:

```
$ printf '123 Main St\nApt 4B\nSpringfield, Illinois 62701\n\n456 Elm Ave\nTrenton, New Jersey\n' | addrnorm --blocks
123 MAIN ST APT 4B, SPRINGFIELD IL 62701
456 ELM AVE, TRENTON NJ
```

The last line of a block is checked for a trailing state and/or zip;
everything before it is joined as the street. If the last line doesn't
resolve to a known state or zip, the whole block is treated as street
with no locality found.

## Deduplication

Pass `--dedupe` to drop lines that normalize to something already seen,
keeping only the first occurrence. It works in both line and `--blocks`
mode, and one seen-set spans every file given on the command line (plus
stdin), so a repeat across files still gets caught:

```
$ printf '123 North Main Street\n123 Main St\n456 Elm Ave\n' | addrnorm --dedupe
123 N MAIN ST
456 ELM AVE
```

## Building

Standard library only, no dependencies:

```
cargo build --release
```

## License

MIT, see LICENSE.
