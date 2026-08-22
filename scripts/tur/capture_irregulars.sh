#!/bin/sh
# Capture the Turkish lemmas the productive rules get wrong with an empty
# exception table — a stable input for scripts/tur/mine_verbs.py, so the
# mine is reproducible rather than chasing a shrinking mismatch file. Run
# once when the rules change; commit scripts/tur/irregulars.txt.
set -e
kept=$(mktemp)
cp data/tur/verbs.tsv "$kept"
printf '# emptied for irregular capture\n' > data/tur/verbs.tsv
cargo run --release --bin golden_tur >/dev/null 2>&1 || true
cp "$kept" data/tur/verbs.tsv
rm -f "$kept"
cut -f1 target/golden_tur_mismatches.tsv | sort -u > scripts/tur/irregulars.txt
wc -l scripts/tur/irregulars.txt
