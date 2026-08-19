#!/bin/sh
# Capture the complete set of Ukrainian lemmas the rule engine gets
# wrong with an empty lexicon: a stable input for scripts/ukr/mine_verbs.py
# so the mine is reproducible rather than chasing a shrinking mismatch
# file. Run once when the rule engine changes; commit scripts/ukr/irregulars.txt.
set -e
cd "$(dirname "$0")/../.."
kept=$(mktemp)
cp data/ukr/verbs.tsv "$kept"
printf '# emptied for irregular capture\n' > data/ukr/verbs.tsv
cargo run --release --bin golden_ukr >/dev/null 2>&1 || true
cut -f1 target/golden_ukr_mismatches.tsv | sort -u > scripts/ukr/irregulars.txt
cp "$kept" data/ukr/verbs.tsv
rm -f "$kept"
wc -l scripts/ukr/irregulars.txt
