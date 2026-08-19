#!/bin/sh
# Capture the complete set of irregular Dutch lemmas: the verbs the
# rule engine gets wrong (or cannot parse) with an empty lexicon. A
# stable input for scripts/nld/mine_verbs.py, so the mine is
# reproducible rather than chasing a shrinking mismatch file. Run once
# when the rule engine changes; commit scripts/nld/irregulars.txt.
set -e
kept=$(mktemp)
cp data/nld/verbs.tsv "$kept"
printf '# emptied for irregular capture\n' > data/nld/verbs.tsv
cargo run --release --bin golden_nld >/dev/null 2>&1 || true
cp "$kept" data/nld/verbs.tsv
rm -f "$kept"
# Rule mismatches, plus every agreed-gold lemma the parser rejects
# (anything not ending in -en: gaan, zijn, doen, zien, …).
{
  cut -f1 target/golden_nld_mismatches.tsv
  cut -f1 data/nld/kaikki.tsv | sort -u > "$kept.ka" 2>/dev/null || true
  comm -12 \
    "$(cut -f1 data/nld/kaikki.tsv | sort -u > /tmp/nld_ka; echo /tmp/nld_ka)" \
    "$(cut -f1 data/nld/apertium.tsv | sort -u > /tmp/nld_ap; echo /tmp/nld_ap)" \
    | grep -v 'en$' || true
} | sort -u > scripts/nld/irregulars.txt
rm -f /tmp/nld_ka /tmp/nld_ap "$kept.ka"
wc -l scripts/nld/irregulars.txt
