#!/bin/sh
# Fetch the OpenCorpora morphological dictionary (CC BY-SA, opencorpora.org)
# and convert its verb paradigms for the Russian golden harness.
#
# OpenCorpora is a hand-checked Russian lexicon of provenance wholly
# independent of Wiktionary — the required second oracle vs kaikki.
#
# Pinned to a checksummed Wayback Machine snapshot (2024-04-23) of the
# canonical dict.opcorpora.txt.zip: the opencorpora.org origin is
# intermittently unavailable, and pinning also stops a silent upstream
# change from shifting the gold standard. The archived bytes are the same
# file curl -sL https://opencorpora.org/files/export/dict/dict.opcorpora.txt.zip
# returns when the origin is up.
set -e
cd "$(dirname "$0")/../.."
mkdir -p data/rus
curl -sL "https://web.archive.org/web/20240423122608id_/https://opencorpora.org/files/export/dict/dict.opcorpora.txt.zip" \
  -o data/rus/dict.opcorpora.txt.zip
echo "032ecf7dc4063de1af00b1eb147759ff47b8a8d45dcc4f5664ee8c8f29901ba5  data/rus/dict.opcorpora.txt.zip" \
  | shasum -a 256 -c -
unzip -o data/rus/dict.opcorpora.txt.zip -d data/rus >/dev/null
python3 scripts/rus/opencorpora_to_tsv.py data/rus/dict.opcorpora.txt > data/rus/opencorpora.tsv
wc -l data/rus/opencorpora.tsv
