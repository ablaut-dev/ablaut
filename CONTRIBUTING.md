# Contributing

## Setup

A stable Rust toolchain is all you need for the library:

```sh
cargo test
```

## The golden harness

Correctness claims are backed by diffing every generated form against gold
data. To reproduce:

```sh
./scripts/deu/fetch_unimorph.sh
cargo run --release --bin golden -- --check     # UniMorph, with CI gates
./scripts/deu/fetch_kaikki.sh                        # larger; optional
cargo run --release --bin golden data/deu/kaikki.tsv
```

Mismatches land in `target/golden_mismatches.tsv`.
`scripts/cross_oracle.py` triages them against the agreement of the two
oracles.

## Fixing a wrong form

1. Reproduce it: `Verb::from_infinitive("...")` in a test, or find the row
   in the mismatch dump.
2. Decide where it belongs:
   - a missing or wrong lexicon ruling: edit [`data/deu/verbs.tsv`](data/deu/verbs.tsv)
     (classes are documented at the top of the file and in
     [`docs/design.md`](docs/design.md));
   - a rule bug: `src/orthography.rs`, `src/prefix.rs` or `src/lib.rs`;
   - the gold data is wrong: add a ruling with a reference to
     [`docs/deu/adjudications.tsv`](docs/deu/adjudications.tsv).
3. Add a unit test and run the harness; CI enforces accuracy gates, so a
   regression elsewhere fails the build.

## Style

`cargo fmt` and `cargo clippy --all-targets -- -D warnings` must pass.
