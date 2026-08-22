#!/bin/sh
# Fetch UniMorph kan (English-Wiktionary lineage, CC BY-SA 3.0; read at
# test time only, never redistributed) and convert it for the Kannada
# golden harness. This is the primary oracle.
#
# Pinned to a commit and checksummed: a silent upstream change would
# shift the gold standard.
set -e
mkdir -p data/kan
curl -sL "https://raw.githubusercontent.com/unimorph/kan/e27276a4de422713b5761ed7e4f695e20cef987d/kan" \
  -o data/kan/unimorph-kan.txt
echo "9ed7830f7f49f7afd2d9843b52a0ba87f2cbdc7360cbffab4b6ab5bbab56987b  data/kan/unimorph-kan.txt" \
  | shasum -a 256 -c -
python3 scripts/kan/unimorph_to_tsv.py data/kan/unimorph-kan.txt > data/kan/unimorph.tsv
wc -l data/kan/unimorph.tsv
