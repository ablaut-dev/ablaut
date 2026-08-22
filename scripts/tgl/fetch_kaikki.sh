#!/bin/sh
# Fetch the kaikki.org Tagalog extraction (Wiktextract of English
# Wiktionary, CC BY-SA; read at test time only, never redistributed) and
# re-key it onto UniMorph tgl's root+trigger schema. This is the second
# leg of the Tagalog oracle pair.
#
# The raw dump is ~120 MB, so only the converted TSV is kept/cached.
set -e
mkdir -p data/tgl
curl -sL "https://kaikki.org/dictionary/Tagalog/kaikki.org-dictionary-Tagalog.jsonl" \
  -o data/tgl/kaikki-tgl.jsonl
python3 scripts/tgl/kaikki_to_tsv.py data/tgl/kaikki-tgl.jsonl > data/tgl/kaikki.tsv
rm -f data/tgl/kaikki-tgl.jsonl
wc -l data/tgl/kaikki.tsv
