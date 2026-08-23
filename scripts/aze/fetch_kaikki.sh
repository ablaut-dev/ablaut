#!/bin/sh
# Fetch the kaikki.org (Wiktextract) Azerbaijani verb extraction (CC BY-SA).
set -e
mkdir -p data/aze
curl -sL "https://kaikki.org/dictionary/Azerbaijani/pos-verb/kaikki.org-dictionary-Azerbaijani-by-pos-verb.jsonl" \
  -o data/aze/kaikki-verbs.jsonl
python3 scripts/aze/kaikki_to_tsv.py data/aze/kaikki-verbs.jsonl > data/aze/kaikki.tsv
wc -l data/aze/kaikki.tsv
