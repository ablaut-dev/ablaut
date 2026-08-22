#!/bin/sh
# Fetch the kaikki.org (Wiktextract) Urdu verb extraction (CC BY-SA) and
# convert it for the Urdu golden harness. One leg of the oracle pair.
# Large: ~11 MB.
set -e
mkdir -p data/urd
curl -sL "https://kaikki.org/dictionary/Urdu/pos-verb/kaikki.org-dictionary-Urdu-by-pos-verb.jsonl" \
  -o data/urd/kaikki-verbs.jsonl
python3 scripts/urd/kaikki_to_tsv.py data/urd/kaikki-verbs.jsonl > data/urd/kaikki.tsv
wc -l data/urd/kaikki.tsv
