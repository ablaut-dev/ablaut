#!/bin/sh
# Fetch BuNaMo — the Irish National Morphology Database (Foras na
# Gaeilge, ODbL).
set -e
mkdir -p data/gle
rm -rf data/gle/bunamo-git
git clone --quiet --depth 1 https://github.com/michmech/BuNaMo data/gle/bunamo-git
python3 scripts/gle/bunamo_to_tsv.py data/gle/bunamo-git > data/gle/bunamo.tsv
rm -rf data/gle/bunamo-git
wc -l data/gle/bunamo.tsv
