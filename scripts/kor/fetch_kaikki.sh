#!/bin/sh
# Fetch the kaikki.org (Wiktextract) Korean verb extraction (CC BY-SA)
# and convert it for the Korean golden harness. Large: ~50 MB.
set -e
mkdir -p data/kor
curl -sL "https://kaikki.org/dictionary/Korean/pos-verb/kaikki.org-dictionary-Korean-by-pos-verb.jsonl" \
  -o data/kor/kaikki-verbs.jsonl
python3 scripts/kor/kaikki_to_tsv.py data/kor/kaikki-verbs.jsonl > data/kor/kaikki.tsv
wc -l data/kor/kaikki.tsv
