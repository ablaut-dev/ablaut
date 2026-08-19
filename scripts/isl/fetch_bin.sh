#!/bin/sh
# Fetch BÍN (Beygingarlýsing íslensks nútímamáls, the Árni Magnússon
# Institute's morphological database of modern Icelandic; CC BY-SA 4.0)
# in the Sigrúnarsnið (SHsnid) CSV format and convert it for the
# Icelandic golden harness. This is the authoritative national lexicon
# and the CI gold — genuinely independent of kaikki (Wiktionary-derived).
#
# The extracted CSV is checksum-pinned against the Institute's own
# published sum: a silent upstream change would shift the gold standard.
set -e
mkdir -p data/isl
curl -sL "https://bin.arnastofnun.is/django/api/nidurhal/?file=SHsnid.csv.zip" \
  -o data/isl/SHsnid.csv.zip
unzip -o -d data/isl data/isl/SHsnid.csv.zip SHsnid.csv >/dev/null
echo "faeaf8289eb287ed3ecff34ae988d4db0c06cb297d1f1e3db41856451581fc23  data/isl/SHsnid.csv" \
  | shasum -a 256 -c -
python3 scripts/isl/bin_to_tsv.py data/isl/SHsnid.csv > data/isl/bin.tsv
wc -l data/isl/bin.tsv
