#!/bin/sh
# Fetch the apertium-nld monodix (GPL-2.0; read at test time only, never
# redistributed) and expand its verb paradigms to the shared TSV. This
# is the independent, non-Wiktionary oracle for the Dutch verification
# loop.
#
# Pinned to a commit and checksummed: a silent upstream change would
# shift the gold standard.
set -e
mkdir -p data/nld
curl -sL "https://raw.githubusercontent.com/apertium/apertium-nld/b429e3b9fbaed116d5cb9cc2b6c4a5bcad94b888/apertium-nld.nld.dix" \
  -o data/nld/apertium-nld.nld.dix
echo "2d14839e0e32f0b057460fa20689060d370eb178b16a51495c00f2b4403250c4  data/nld/apertium-nld.nld.dix" \
  | shasum -a 256 -c -
python3 scripts/nld/apertium_to_tsv.py data/nld/apertium-nld.nld.dix > data/nld/apertium.tsv
wc -l data/nld/apertium.tsv
