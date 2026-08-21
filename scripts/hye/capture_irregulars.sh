#!/bin/sh
# Capture the Armenian lemmas the rule engine gets wrong with an empty
# table — a stable input for scripts/hye/mine_verbs.py, so the mine is
# reproducible rather than chasing a shrinking mismatch file. Run once
# when the rule engine changes; commit scripts/hye/irregulars.txt.
set -e
kept=$(mktemp)
cp data/hye/verbs.tsv "$kept"
printf '# emptied for irregular capture\n' > data/hye/verbs.tsv
cargo run --release --bin golden_hye >/dev/null 2>&1 || true
cp "$kept" data/hye/verbs.tsv
rm -f "$kept"
cut -f1 target/golden_hye_mismatches.tsv | sort -u > scripts/hye/irregulars.txt
wc -l scripts/hye/irregulars.txt
