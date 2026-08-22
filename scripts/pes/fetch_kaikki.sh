#!/bin/sh
# Fetch the kaikki.org (Wiktextract) Persian verb extraction (CC BY-SA;
# read at test time only, never redistributed) and convert it to the
# common golden TSV. This is the Wiktionary leg of the oracle pair.
# Large: ~45 MB. Not commit-pinnable (kaikki rebuilds in place); the
# converter is defensive about what it accepts.
set -e
mkdir -p data/pes
curl -sL "https://kaikki.org/dictionary/Persian/pos-verb/kaikki.org-dictionary-Persian-by-pos-verb.jsonl" \
  -o data/pes/kaikki-verbs.jsonl
python3 scripts/pes/kaikki_to_tsv.py data/pes/kaikki-verbs.jsonl > data/pes/kaikki.tsv
wc -l data/pes/kaikki.tsv
