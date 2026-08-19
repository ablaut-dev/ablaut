#!/bin/sh
# Fetch the mecab-ipadic 2.7.0 source dictionary (the second, non-Wiktionary
# oracle), extract its verb CSV, transcode EUC-JP → UTF-8 and convert it to
# the second-oracle TSV. IPADIC descends from the Kyoto/NAIST corpus work,
# independent of Wiktionary.
#
# The canonical tarball lives on a Google Drive that curl cannot fetch
# unattended; this uses the byte-identical mirror bundled with the
# shogo82148/mecab release (same 2.7.0-20070801 dictionary).
set -e
mkdir -p data/jpn
TMP="$(mktemp -d)"
URL="https://github.com/shogo82148/mecab/releases/download/v0.996.10/mecab-ipadic-2.7.0-20070801.tar.gz"
curl -sSL "$URL" -o "$TMP/ipadic.tar.gz"
tar xzf "$TMP/ipadic.tar.gz" -C "$TMP"
iconv -f EUC-JP -t UTF-8 "$TMP"/mecab-ipadic-2.7.0-20070801/Verb.csv > /tmp/ipa/Verb.utf8.csv 2>/dev/null \
  || { mkdir -p /tmp/ipa; iconv -f EUC-JP -t UTF-8 "$TMP"/mecab-ipadic-2.7.0-20070801/Verb.csv > /tmp/ipa/Verb.utf8.csv; }
python3 scripts/jpn/ipadic_to_tsv.py /tmp/ipa/Verb.utf8.csv > data/jpn/ipadic.tsv
wc -l data/jpn/ipadic.tsv
rm -rf "$TMP"
