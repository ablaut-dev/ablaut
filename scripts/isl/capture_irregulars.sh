#!/bin/sh
# Capture the complete set of irregular Icelandic lemmas: the verbs the
# ō-class rule engine gets wrong with an empty lexicon. A stable input
# for scripts/isl/mine_verbs.py, so the mine is reproducible rather than
# chasing a shrinking mismatch file. Run once when the rule engine
# changes; commit scripts/isl/irregulars.txt.
set -e
kept=$(mktemp)
cp data/isl/verbs.tsv "$kept"
printf '# emptied for irregular capture\n' > data/isl/verbs.tsv
# Score against BÍN alone (all lemmas, not just the ~1,900 the two
# oracles share) so BÍN-only irregulars are captured too.
cargo run --release --bin golden_isl -- data/isl/bin.tsv >/dev/null 2>&1 || true
cut -f1 target/golden_isl_mismatches.tsv | sort -u > scripts/isl/irregulars.txt
cp "$kept" data/isl/verbs.tsv
rm -f "$kept"
wc -l scripts/isl/irregulars.txt
