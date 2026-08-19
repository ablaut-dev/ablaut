#!/bin/sh
# Fetch the NIKL 한국어기초사전 (Korean Basic Dictionary) LMF-XML dump and
# convert its 활용 (conjugation) fields for the Korean golden harness.
#
# The dictionary itself is CC-BY-SA 2.0 KR, published by the National
# Institute of Korean Language and downloadable (login-free) from
# krdict.korean.go.kr. We mirror it from the community archive
# spellcheck-ko/korean-dict-nikl-krdict, commit-pinned for reproducibility.
# Large: ~400 MB of XML.
set -e
COMMIT=e7249562dca526461a347eb2cc5a2e67a17c4488
mkdir -p data/kor
rm -rf data/kor/krdict-xml
git clone --quiet https://github.com/spellcheck-ko/korean-dict-nikl-krdict.git \
  data/kor/krdict-xml
git -C data/kor/krdict-xml checkout --quiet "$COMMIT"
python3 scripts/kor/krdict_to_tsv.py "data/kor/krdict-xml/*.xml" > data/kor/krdict.tsv
wc -l data/kor/krdict.tsv
