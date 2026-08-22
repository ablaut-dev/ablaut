#!/bin/sh
# Fetch UniMorph tgl (English-Wiktionary lineage, CC BY-SA 3.0; read at
# test time only, never redistributed) and convert it for the Tagalog
# golden harness. This is one leg of the Tagalog oracle pair.
#
# Pinned by checksum: a silent upstream change would shift the gold.
set -e
mkdir -p data/tgl
curl -sL "https://raw.githubusercontent.com/unimorph/tgl/master/tgl" \
  -o data/tgl/unimorph-tgl.txt
echo "84bcc773fde9430d9c8f9d10c72cd4ebca6410eac0eb8408accf0fd6b49e840b  data/tgl/unimorph-tgl.txt" \
  | shasum -a 256 -c -
python3 scripts/tgl/unimorph_to_tsv.py data/tgl/unimorph-tgl.txt > data/tgl/unimorph.tsv
wc -l data/tgl/unimorph.tsv
