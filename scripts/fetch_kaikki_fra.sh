#!/bin/sh
# Fetch the kaikki.org (Wiktextract) French verb extraction (CC BY-SA) and
# convert it for the French golden harness.
set -e
mkdir -p data/kaikki
curl -sL "https://kaikki.org/dictionary/French/pos-verb/kaikki.org-dictionary-French-by-pos-verb.jsonl" \
  -o data/kaikki/verbs-fra.jsonl
python3 scripts/kaikki_fra_to_tsv.py data/kaikki/verbs-fra.jsonl > data/kaikki/fra.tsv
wc -l data/kaikki/fra.tsv
