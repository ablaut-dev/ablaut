#!/bin/sh
# Fetch the kaikki.org (Wiktextract) Portuguese verb extraction
# (CC BY-SA) and convert it for the Portuguese golden harness.
set -e
mkdir -p data/por
curl -sL "https://kaikki.org/dictionary/Portuguese/pos-verb/kaikki.org-dictionary-Portuguese-by-pos-verb.jsonl" \
  -o data/por/kaikki-verbs.jsonl
python3 scripts/por/kaikki_to_tsv.py data/por/kaikki-verbs.jsonl > data/por/kaikki.tsv
wc -l data/por/kaikki.tsv
