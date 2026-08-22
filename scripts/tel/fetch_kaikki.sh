#!/bin/sh
# Fetch the kaikki.org Telugu extraction (Wiktextract of English
# Wiktionary, CC BY-SA; read at test time only, never redistributed)
# and convert its verb inflection tables for the Telugu harness.
#
# This is the intended independent second oracle, but Telugu Wiktionary
# fills conjugation tables for only a handful of verbs (see
# docs/tel/oracles.md), so kaikki serves as a spot check rather than an
# agreement partner. Not commit-pinnable (kaikki rebuilds in place); the
# converter is defensive about what it accepts.
set -e
mkdir -p data/tel
curl -sL "https://kaikki.org/dictionary/Telugu/kaikki.org-dictionary-Telugu.jsonl" \
  -o data/tel/kaikki-telugu.jsonl
python3 scripts/tel/kaikki_to_tsv.py data/tel/kaikki-telugu.jsonl > data/tel/kaikki.tsv
wc -l data/tel/kaikki.tsv
