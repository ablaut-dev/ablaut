#!/bin/sh
# Fetch the kaikki.org (Wiktextract) Belarusian verb extraction (CC BY-SA).
set -e
mkdir -p data/bel
curl -sL "https://kaikki.org/dictionary/Belarusian/pos-verb/kaikki.org-dictionary-Belarusian-by-pos-verb.jsonl" \
  -o data/bel/kaikki-verbs.jsonl
python3 scripts/bel/kaikki_to_tsv.py data/bel/kaikki-verbs.jsonl > data/bel/kaikki.tsv
wc -l data/bel/kaikki.tsv
