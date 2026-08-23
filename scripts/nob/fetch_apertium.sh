#!/bin/sh
# Build the independent oracle from apertium-nob: clone the monolingual
# Bokmål package (commit-pinned) and expand its lttoolbox dictionary into
# every surface/analysis pair with `lt-expand`, then map the analyses onto
# the shared UniMorph-style bundle. apertium-nob is a hand-built FST
# dictionary with no Wiktionary lineage, so it is the independent leg of
# the pair.
#
# Requires lttoolbox (`lt-expand`) on PATH.
set -e
mkdir -p data/nob
REPO="https://github.com/apertium/apertium-nob"
PIN="master"
DIR="data/nob/apertium-nob"
if [ ! -d "$DIR" ]; then
  git clone --depth 1 "$REPO" "$DIR"
fi
lt-expand "$DIR/apertium-nob.nob.dix" \
  | python3 scripts/nob/apertium_to_tsv.py > data/nob/apertium.tsv
wc -l data/nob/apertium.tsv
