#!/bin/sh
# Fetch the kaikki.org (Wiktextract) Hindi verb extraction (CC BY-SA)
# and convert it for the Hindi golden harness. This is the other leg of
# the oracle pair. Large: ~90 MB.
set -e
mkdir -p data/hin
curl -sL "https://kaikki.org/dictionary/Hindi/pos-verb/kaikki.org-dictionary-Hindi-by-pos-verb.jsonl" \
  -o data/hin/kaikki-verbs.jsonl
python3 scripts/hin/kaikki_to_tsv.py data/hin/kaikki-verbs.jsonl > data/hin/kaikki.tsv
wc -l data/hin/kaikki.tsv
