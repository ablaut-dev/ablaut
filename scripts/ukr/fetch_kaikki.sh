#!/bin/sh
# Fetch the kaikki.org (Wiktextract) Ukrainian verb extraction (CC BY-SA)
# and convert it for the Ukrainian golden harness. Large: ~80 MB.
set -e
cd "$(dirname "$0")/../.."
mkdir -p data/ukr
curl -sL "https://kaikki.org/dictionary/Ukrainian/pos-verb/kaikki.org-dictionary-Ukrainian-by-pos-verb.jsonl" \
  -o data/ukr/kaikki-verbs.jsonl
python3 scripts/ukr/kaikki_to_tsv.py data/ukr/kaikki-verbs.jsonl > data/ukr/kaikki.tsv
wc -l data/ukr/kaikki.tsv
