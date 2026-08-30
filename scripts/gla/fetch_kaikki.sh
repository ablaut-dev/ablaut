#!/bin/sh
# Fetch the kaikki.org (Wiktextract) Scottish Gaelic verb extraction
# (CC BY-SA) and convert it into the golden-harness gold
# (data/gla/kaikki.tsv) plus the mined principal parts
# (data/gla/verbs.tsv). The single kaikki oracle places Scottish Gaelic
# in the Beta tier. Forms carry their mutations/particles baked in
# (ghlan, chuir, dh'òl); the converter normalizes them to unmutated
# citation forms, exactly as the Irish pipeline does.
set -e
mkdir -p data/gla
curl -sL "https://kaikki.org/dictionary/Scottish%20Gaelic/pos-verb/kaikki.org-dictionary-ScottishGaelic-by-pos-verb.jsonl" \
  -o data/gla/kaikki.jsonl
python3 scripts/gla/kaikki_to_tsv.py data/gla/kaikki.jsonl
wc -l data/gla/kaikki.tsv
