#!/bin/sh
# Fetch the kaikki.org (Wiktextract) Dutch verb extraction (CC BY-SA)
# and convert it for the Dutch golden harness. Large: ~88 MB.
set -e
mkdir -p data/nld
curl -sL "https://kaikki.org/dictionary/Dutch/pos-verb/kaikki.org-dictionary-Dutch-by-pos-verb.jsonl" \
  -o data/nld/kaikki-verbs.jsonl
python3 scripts/nld/kaikki_to_tsv.py data/nld/kaikki-verbs.jsonl > data/nld/kaikki.tsv
wc -l data/nld/kaikki.tsv
