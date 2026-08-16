#!/bin/sh
# Fetch the kaikki.org (Wiktextract) French verb extraction (CC BY-SA) and
# convert it for the French golden harness.
set -e
mkdir -p data/fra
curl -sL "https://kaikki.org/dictionary/French/pos-verb/kaikki.org-dictionary-French-by-pos-verb.jsonl" \
  -o data/fra/kaikki-verbs.jsonl
python3 scripts/fra/kaikki_to_tsv.py data/fra/kaikki-verbs.jsonl > data/fra/kaikki.tsv
wc -l data/fra/kaikki.tsv
