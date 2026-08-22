#!/bin/sh
# Build the independent second oracle from apertium-urd: clone the
# monolingual Urdu package (commit-pinned) and expand its lttoolbox
# dictionary into every surface/analysis pair, then map the analyses onto
# the shared feature bundle. apertium-urd is a hand-built dictionary with
# no Wiktionary lineage, so its agreement with kaikki is the two-oracle
# gate.
#
# The .dix is expanded directly by scripts/urd/apertium_to_tsv.py (the
# monodix is flat), so no lttoolbox install is needed.
set -e
mkdir -p data/urd
REPO="https://github.com/apertium/apertium-urd"
PIN="8d83e85d124c20697d1a44f0089a93fe34777c5a"
DIR="data/urd/apertium-urd"
if [ ! -d "$DIR" ]; then
  git clone "$REPO" "$DIR"
fi
git -C "$DIR" fetch --depth 1 origin "$PIN"
git -C "$DIR" checkout -q "$PIN"
python3 scripts/urd/apertium_to_tsv.py "$DIR/apertium-urd.urd.dix" > data/urd/apertium.tsv
wc -l data/urd/apertium.tsv
