#!/bin/sh
# Fetch the kaikki.org Bengali verb extraction (Wiktextract of English
# Wiktionary, CC BY-SA; read at test time only, never redistributed) and
# convert it for the Bengali harness.
#
# kaikki ben is the *same* Wiktionary lineage as UniMorph ben (whose
# source is Wikipedia), so it is not an independent agreement partner —
# it serves as a spot check (see docs/ben/oracles.md). kaikki also cannot
# tell তুই (LGSPEC1) from তুমি (3;INFM): both carry
# `familiar;second-person`, so the second-person-familiar cells are
# dropped and only the unambiguous person classes are recovered. Not
# commit-pinnable (kaikki rebuilds in place).
set -e
mkdir -p data/ben
curl -sL "https://kaikki.org/dictionary/Bengali/pos-verb/kaikki.org-dictionary-Bengali-by-pos-verb.jsonl" \
  -o data/ben/kaikki-verbs.jsonl
python3 scripts/ben/kaikki_to_tsv.py data/ben/kaikki-verbs.jsonl > data/ben/kaikki.tsv
wc -l data/ben/kaikki.tsv
