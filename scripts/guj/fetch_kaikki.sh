#!/bin/sh
# Fetch the kaikki.org Gujarati verb extraction (Wiktextract of English
# Wiktionary, CC BY-SA; read at test time only, never redistributed) and
# convert it for the Gujarati harness.
#
# kaikki guj is the *same* English-Wiktionary lineage as UniMorph guj
# (both descend from the `gu-conj` template), so it is not an independent
# agreement partner — it serves as a spot check (see docs/guj/oracles.md).
# Wiktextract could not map the Gujarati column headers, so many cells
# are tagged `error-unrecognized-form`; the converter recovers only the
# categories it can read unambiguously. Not commit-pinnable (kaikki
# rebuilds in place).
set -e
mkdir -p data/guj
curl -sL "https://kaikki.org/dictionary/Gujarati/pos-verb/kaikki.org-dictionary-Gujarati-by-pos-verb.jsonl" \
  -o data/guj/kaikki-verbs.jsonl
python3 scripts/guj/kaikki_to_tsv.py data/guj/kaikki-verbs.jsonl > data/guj/kaikki.tsv
wc -l data/guj/kaikki.tsv
