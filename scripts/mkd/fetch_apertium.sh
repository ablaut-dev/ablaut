#!/bin/sh
# Build the independent oracle from apertium-mkd: clone the monolingual
# Macedonian package and expand its lttoolbox dictionary into every
# surface/analysis pair with `lt-expand`, mapped onto the shared bundle.
# apertium-mkd is a hand-built FST dictionary with no Wiktionary lineage,
# so it is the independent leg of the pair. Requires lttoolbox-dev
# (`lt-expand`).
set -e
mkdir -p data/mkd
REPO="https://github.com/apertium/apertium-mkd"
DIR="data/mkd/apertium-mkd"
if [ ! -d "$DIR" ]; then
  git clone --depth 1 "$REPO" "$DIR"
fi
lt-expand "$DIR/apertium-mkd.mkd.dix" \
  | python3 scripts/mkd/apertium_to_tsv.py > data/mkd/apertium.tsv
wc -l data/mkd/apertium.tsv
