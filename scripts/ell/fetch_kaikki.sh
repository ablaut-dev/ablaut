#!/bin/sh
# Fetch the kaikki.org (Wiktextract) Greek verb extraction (CC BY-SA).
set -e
mkdir -p data/ell
curl -sL "https://kaikki.org/dictionary/Greek/pos-verb/kaikki.org-dictionary-Greek-by-pos-verb.jsonl" \
  -o data/ell/kaikki-verbs.jsonl
python3 scripts/ell/kaikki_to_tsv.py data/ell/kaikki-verbs.jsonl > data/ell/kaikki.tsv
wc -l data/ell/kaikki.tsv
