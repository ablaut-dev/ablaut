#!/bin/sh
# Fetch the kaikki.org (Wiktextract) Turkmen verb extraction (CC BY-SA).
set -e
mkdir -p data/tuk
curl -sL "https://kaikki.org/dictionary/Turkmen/pos-verb/kaikki.org-dictionary-Turkmen-by-pos-verb.jsonl" \
  -o data/tuk/kaikki-verbs.jsonl
python3 scripts/tuk/kaikki_to_tsv.py data/tuk/kaikki-verbs.jsonl > data/tuk/kaikki.tsv
wc -l data/tuk/kaikki.tsv
