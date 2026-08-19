#!/bin/sh
# Capture the complete set of irregular Catalan lemmas: the verbs the
# rule engine gets wrong with an empty lexicon. A stable input for
# scripts/cat/mine_verbs.py, so the mine is reproducible rather than
# chasing a shrinking mismatch file. Run once when the rule engine
# changes; commit scripts/cat/irregulars.txt.
set -e
kept=$(mktemp)
cp data/cat/verbs.tsv "$kept"
printf '# emptied for irregular capture\n' > data/cat/verbs.tsv
# Score against FreeLing alone (all 8,722 lemmas, not just the 3,750 the
# two oracles share) so FreeLing-only irregulars are captured too.
cargo run --release --bin golden_cat -- data/cat/freeling.tsv >/dev/null 2>&1 || true
cut -f1 target/golden_cat_mismatches.tsv | sort -u > scripts/cat/irregulars.txt
cp "$kept" data/cat/verbs.tsv
rm -f "$kept"
wc -l scripts/cat/irregulars.txt
