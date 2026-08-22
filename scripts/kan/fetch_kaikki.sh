#!/bin/sh
# Fetch the kaikki.org Kannada extraction (Wiktextract of English
# Wiktionary, CC BY-SA; read at test time only, never redistributed)
# and convert its verb inflection tables for the Kannada harness. This
# is the independent second oracle.
#
# Unlike Telugu, the Kannada Wiktextract mapped its column headers
# cleanly: ~165 verbs carry a full person/number/gender/tense-tagged
# table, giving a real two-oracle agreement surface against UniMorph.
# Not commit-pinnable (kaikki rebuilds in place); the converter is
# defensive about what it accepts (it drops malformed extractions).
set -e
mkdir -p data/kan
curl -sL "https://kaikki.org/dictionary/Kannada/kaikki.org-dictionary-Kannada.jsonl" \
  -o data/kan/kaikki-kannada.jsonl
python3 scripts/kan/kaikki_to_tsv.py data/kan/kaikki-kannada.jsonl > data/kan/kaikki.tsv
wc -l data/kan/kaikki.tsv
