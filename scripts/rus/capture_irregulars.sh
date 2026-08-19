#!/bin/sh
# Capture the complete set of irregular Russian lemmas: the verbs the
# rule engine gets wrong with an empty lexicon. A stable input for
# scripts/rus/mine_verbs.py, so the mine is reproducible rather than
# chasing a shrinking mismatch file. Run once when the rule engine
# changes; commit scripts/rus/irregulars.txt.
set -e
cd "$(dirname "$0")/../.."
kept=$(mktemp)
cp data/rus/verbs.tsv "$kept"
printf '# emptied for irregular capture\n' > data/rus/verbs.tsv
# Score against OpenCorpora alone (all ~31k lemmas, not just the ~11.6k
# the two oracles share) so OpenCorpora-only irregulars are captured too
# — that is the file the CI gate scores against.
cargo run --release --bin golden_rus -- data/rus/opencorpora.tsv >/dev/null 2>&1 || true
cut -f1 target/golden_rus_mismatches.tsv | sort -u > scripts/rus/irregulars.txt
cp "$kept" data/rus/verbs.tsv
rm -f "$kept"
wc -l scripts/rus/irregulars.txt
