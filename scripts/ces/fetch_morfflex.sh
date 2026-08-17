#!/bin/sh
# Fetch MorfFlex CZ 2.1 (LINDAT/CLARIAH-CZ, CC BY-NC-SA — test-time
# oracle only, nothing ships). 250 MB xz, streamed through the
# converter.
set -e
mkdir -p data/ces
curl -sL "https://lindat.mff.cuni.cz/repository/server/api/core/bitstreams/cc940076-7bf0-449b-81a4-5e10e023084e/content" \
  -o data/ces/morfflex.tsv.xz
xz -dc data/ces/morfflex.tsv.xz | python3 scripts/ces/morfflex_to_tsv.py - > data/ces/morfflex.tsv
wc -l data/ces/morfflex.tsv
