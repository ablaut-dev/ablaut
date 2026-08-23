#!/bin/sh
# Fetch the kaikki.org (Wiktextract) Occitan verb extraction (CC BY-SA).
set -e
mkdir -p data/oci
curl -sL "https://kaikki.org/dictionary/Occitan/pos-verb/kaikki.org-dictionary-Occitan-by-pos-verb.jsonl" \
  -o data/oci/kaikki-verbs.jsonl
python3 scripts/oci/kaikki_to_tsv.py data/oci/kaikki-verbs.jsonl > data/oci/kaikki.tsv
wc -l data/oci/kaikki.tsv
