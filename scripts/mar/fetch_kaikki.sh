#!/bin/sh
# Fetch the kaikki.org Marathi verb extraction (Wiktextract of English
# Wiktionary, CC BY-SA; read at test time only, never redistributed) and
# convert it for the Marathi harness.
#
# kaikki Marathi is the independent second oracle, but Wiktextract lost
# the person/number on its finite mr-conj cells, so its clean per-cell
# contribution is the non-finite forms (see docs/mar/oracles.md). Not
# commit-pinnable (kaikki rebuilds in place).
set -e
mkdir -p data/mar
curl -sL "https://kaikki.org/dictionary/Marathi/pos-verb/kaikki.org-dictionary-Marathi-by-pos-verb.jsonl" \
  -o data/mar/kaikki-verbs.jsonl
python3 scripts/mar/kaikki_to_tsv.py data/mar/kaikki-verbs.jsonl > data/mar/kaikki.tsv
wc -l data/mar/kaikki.tsv
