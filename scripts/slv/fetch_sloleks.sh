#!/bin/sh
# Fetch Sloleks 3.0 (CJVT; CC BY-SA 4.0) via the Hugging Face SQLite
# mirror — CLARIN.SI's own bitstreams for Sloleks 2.0 and 3.0 both
# return server-side I/O errors (2026-08-17).
set -e
mkdir -p data/slv
curl -sL "https://huggingface.co/datasets/ppisljar/sloleks-3-sql/resolve/main/sloleks.db" \
  -o data/slv/sloleks.db
python3 scripts/slv/sloleks_to_tsv.py data/slv/sloleks.db > data/slv/sloleks.tsv
wc -l data/slv/sloleks.tsv
