#!/usr/bin/env bash
# Fetch the kaikki.org Paraguayan Guarani dump used to build the gold
# files. Run from the repo root, then `python3 scripts/grn/build.py`.
set -euo pipefail
mkdir -p data/grn
URL="https://kaikki.org/dictionary/Paraguayan%20Guarani/kaikki.org-dictionary-ParaguayanGuarani.jsonl"
curl -fSL "$URL" -o data/grn/kaikki-raw.jsonl
wc -l data/grn/kaikki-raw.jsonl
