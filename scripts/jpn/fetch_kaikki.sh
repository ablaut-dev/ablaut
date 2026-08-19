#!/bin/sh
# Fetch the kaikki.org (Wiktextract) Japanese verb extraction (CC BY-SA)
# and convert it to the first-oracle TSV. Large: ~42 MB.
set -e
mkdir -p data/jpn
curl -sL "https://kaikki.org/dictionary/Japanese/pos-verb/kaikki.org-dictionary-Japanese-by-pos-verb.jsonl" \
  -o data/jpn/kaikki-verbs.jsonl
python3 scripts/jpn/kaikki_to_tsv.py data/jpn/kaikki-verbs.jsonl > data/jpn/kaikki.tsv
wc -l data/jpn/kaikki.tsv
