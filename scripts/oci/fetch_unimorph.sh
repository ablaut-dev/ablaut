#!/bin/sh
# Fetch the UniMorph Occitan inflection table (CC BY-SA), keep verbs.
set -e
mkdir -p data/oci
curl -sL "https://raw.githubusercontent.com/unimorph/oci/master/oci" -o data/oci/unimorph-oci.tsv
awk -F'\t' '$3 ~ /^V/' data/oci/unimorph-oci.tsv > data/oci/unimorph.tsv
wc -l data/oci/unimorph.tsv
