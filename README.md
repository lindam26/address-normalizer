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

Blank lines are skipped. Each input line is treated as one address line -
this version does not parse a full multi-line address block into
street/city/state/zip fields yet.

## Building

Standard library only, no dependencies:

```
cargo build --release
```

## License

MIT, see LICENSE.
