#!/bin/sh
# Fetch the dexonline database dump (GPL — used as a test-time oracle
# only, nothing ships) and extract the verb inflections. The dump is
# regenerated upstream regularly, so no checksum pin; the golden gate
# margin absorbs routine dictionary growth.
set -e
mkdir -p data/ron
curl -sL "https://dexonline.ro/static/download/dex-database.sql.gz" \
  -o data/ron/dex-database.sql.gz
python3 scripts/ron/dexonline_to_tsv.py data/ron/dex-database.sql.gz > data/ron/dexonline.tsv
wc -l data/ron/dexonline.tsv
