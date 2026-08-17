#!/bin/sh
# Fetch the kaikki.org (Wiktextract) Finnish verb extraction (CC BY-SA).
set -e
mkdir -p data/fin
curl -sL "https://kaikki.org/dictionary/Finnish/pos-verb/kaikki.org-dictionary-Finnish-by-pos-verb.jsonl" \
  -o data/fin/kaikki-verbs.jsonl
python3 scripts/fin/kaikki_to_tsv.py data/fin/kaikki-verbs.jsonl > data/fin/kaikki.tsv
wc -l data/fin/kaikki.tsv
