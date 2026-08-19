#!/bin/sh
# Fetch the kaikki.org (Wiktextract) Russian verb extraction (CC BY-SA)
# and convert it for the Russian golden harness. Large: ~440 MB.
set -e
cd "$(dirname "$0")/../.."
mkdir -p data/rus
curl -sL "https://kaikki.org/dictionary/Russian/pos-verb/kaikki.org-dictionary-Russian-by-pos-verb.jsonl" \
  -o data/rus/kaikki-verbs.jsonl
python3 scripts/rus/kaikki_to_tsv.py data/rus/kaikki-verbs.jsonl > data/rus/kaikki.tsv
wc -l data/rus/kaikki.tsv
