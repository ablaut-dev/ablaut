#!/bin/sh
# Build the primary oracle from apertium-mar: clone the monolingual
# Marathi package (commit-pinned) and expand its lttoolbox dictionary
# into every surface/analysis pair with `lt-expand`, then map the
# analyses onto the shared feature bundle.
#
# Requires lttoolbox (`lt-expand`) on PATH — `apt-get install lttoolbox`
# on Debian/Ubuntu, or a source build. apertium-mar is a hand-built FST
# dictionary with no Wiktionary lineage, so it is the independent leg of
# the pair and the full-paradigm scoring oracle.
set -e
mkdir -p data/mar
REPO="https://github.com/apertium/apertium-mar"
PIN="959f483463fee06ccf0895bb216f727a609b1087"
DIR="data/mar/apertium-mar"
if [ ! -d "$DIR" ]; then
  git clone "$REPO" "$DIR"
fi
git -C "$DIR" fetch --depth 1 origin "$PIN"
git -C "$DIR" checkout -q "$PIN"
lt-expand "$DIR/apertium-mar.mar.dix" \
  | python3 scripts/mar/apertium_to_tsv.py > data/mar/apertium.tsv
wc -l data/mar/apertium.tsv
