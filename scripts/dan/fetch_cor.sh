#!/bin/sh
# Fetch COR — Det Centrale Ordregister (Dansk Sprognævn; open data).
set -e
mkdir -p data/dan
curl -sL "https://ordregister.dk/files/cor1.02.tsv" -o data/dan/cor.tsv
python3 scripts/dan/cor_to_tsv.py data/dan/cor.tsv > data/dan/cor-verbs.tsv
wc -l data/dan/cor-verbs.tsv
