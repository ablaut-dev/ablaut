#!/bin/sh
# Build the independent second oracle from apertium-pes: clone the
# monolingual Persian package (commit-pinned) and expand its lttoolbox
# dictionary into every surface/analysis pair with `lt-expand`, then map
# the analyses onto the shared feature bundle.
#
# Requires lttoolbox (`lt-expand`) on PATH — `apt-get install lttoolbox`
# on Debian/Ubuntu, or a source build. apertium-pes is a hand-built FST
# dictionary with no Wiktionary lineage, so its agreement with kaikki is
# the two-oracle gate.
set -e
mkdir -p data/pes
REPO="https://github.com/apertium/apertium-pes"
PIN="16757db6a56aca9ab8b7c9391ebbaf8c1f939604"
DIR="data/pes/apertium-pes"
if [ ! -d "$DIR" ]; then
  git clone "$REPO" "$DIR"
fi
git -C "$DIR" fetch --depth 1 origin "$PIN"
git -C "$DIR" checkout -q "$PIN"
lt-expand "$DIR/apertium-pes.pes.dix" \
  | python3 scripts/pes/apertium_to_tsv.py > data/pes/apertium.tsv
wc -l data/pes/apertium.tsv
