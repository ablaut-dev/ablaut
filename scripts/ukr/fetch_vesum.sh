#!/bin/sh
# Fetch the VESUM (brown-uk/dict_uk) expanded full-form dictionary, the
# independent non-Wiktionary second oracle for Ukrainian. Release-pinned
# and sha256-checked. The bz2 is ~18 MB and expands to ~320 MB.
set -e
cd "$(dirname "$0")/../.."
mkdir -p data/ukr
URL="https://github.com/brown-uk/dict_uk/releases/download/v6.8.0/dict_corp_vis.txt.bz2"
SHA="e33803783ac138e6f3af2cf0e9428ba146c0ecfda7f5c41fe83ae00c7af24be9"
curl -sL "$URL" -o data/ukr/dict_corp_vis.txt.bz2
echo "$SHA  data/ukr/dict_corp_vis.txt.bz2" | shasum -a 256 -c -
bunzip2 -kf data/ukr/dict_corp_vis.txt.bz2
python3 scripts/ukr/vesum_to_tsv.py data/ukr/dict_corp_vis.txt > data/ukr/vesum.tsv
wc -l data/ukr/vesum.tsv
