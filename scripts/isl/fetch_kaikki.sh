#!/bin/sh
# Fetch the kaikki.org (Wiktextract) Icelandic verb extraction (CC BY-SA)
# and convert it for the Icelandic golden harness. ~12 MB.
#
# kaikki's Icelandic verb tables extract only partially — the finite
# paradigm reduces to the third-person past and the supine, plus the
# declined participle table — so kaikki serves as the independent
# provenance check on BÍN (see docs/isl/oracles.md), not the CI gate.
set -e
mkdir -p data/isl
curl -sL "https://kaikki.org/dictionary/Icelandic/pos-verb/kaikki.org-dictionary-Icelandic-by-pos-verb.jsonl" \
  -o data/isl/kaikki-verbs.jsonl
python3 scripts/isl/kaikki_to_tsv.py data/isl/kaikki-verbs.jsonl > data/isl/kaikki.tsv
wc -l data/isl/kaikki.tsv
