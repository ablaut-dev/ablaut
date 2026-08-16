#!/bin/sh
# Fetch the UniMorph German dataset (CC BY-SA 3.0) used by the golden harness:
#   cargo run --release --bin golden
set -e
mkdir -p data/deu
curl -sL https://raw.githubusercontent.com/unimorph/deu/master/deu -o data/deu/unimorph
wc -l data/deu/unimorph
