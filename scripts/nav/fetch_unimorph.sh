#!/usr/bin/env bash
# Fetch UniMorph Navajo and keep only the verb paradigm (V;...), the scoped oracle.
set -euo pipefail
DIR="$(cd "$(dirname "$0")/../../data/nav" && pwd)"
URL="https://raw.githubusercontent.com/unimorph/nav/master/nav"
curl -sL "$URL" | awk -F'\t' '$3 ~ /^V/' | sort -u > "$DIR/unimorph.tsv"
echo "wrote $DIR/unimorph.tsv ($(wc -l < "$DIR/unimorph.tsv") verb triples)"
