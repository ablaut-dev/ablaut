#!/bin/sh
# Fetch the kaikki.org (Wiktextract) Catalan verb extraction (CC BY-SA)
# and convert it for the Catalan golden harness. Large: ~150 MB.
set -e
mkdir -p data/cat
curl -sL "https://kaikki.org/dictionary/Catalan/pos-verb/kaikki.org-dictionary-Catalan-by-pos-verb.jsonl" \
  -o data/cat/kaikki-verbs.jsonl
python3 scripts/cat/kaikki_to_tsv.py data/cat/kaikki-verbs.jsonl > data/cat/kaikki.tsv
wc -l data/cat/kaikki.tsv
