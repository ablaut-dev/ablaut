#!/usr/bin/env python3
"""Convert UniMorph tur to the shared `lemma ⇥ form ⇥ features` TSV.

UniMorph tur is generated from TRmorph (a finite-state Turkish
morphology), native-verified, and enumerates a very large paradigm:
voice × TAM × person × number × polarity × interrogative × copular
stacking runs to ~413k verb forms over 588 lemmas. The Turkish verb is
agglutinative and almost every cell is one word, so this converter keeps
the single-word forms and drops the periphrastic ones.

What is kept, and why it is a well-defined bound:

* only single-word forms (no spaces) — the periphrastic tenses
  (`gönderecek olacak`) and the interrogative (`gönderir mi`, always the
  free particle `mi`) are syntax, out of scope;
* the declarative mood only — `;INTR` is the `mi` particle, dropped with
  the multi-word forms above;
* `;DECL` is stripped from the feature bundle (it is redundant once the
  interrogative is gone), leaving UniMorph's own bundles otherwise
  intact so the kaikki leg can be aligned to them.

Usage: python3 scripts/tur/unimorph_to_tsv.py data/tur/unimorph-tur.tsv
"""

import sys


def main(path):
    rows = set()
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            fields = line.rstrip("\n").split("\t")
            if len(fields) != 3:
                continue
            lemma, form, feats = (f.strip() for f in fields)
            if not feats.startswith("V"):
                continue
            if not form or " " in form:
                continue
            tags = feats.split(";")
            if "INTR" in tags:
                continue
            tags = [t for t in tags if t != "DECL"]
            rows.add((lemma, form, ";".join(tags)))
    out = sys.stdout
    for lemma, form, feats in sorted(rows):
        out.write(f"{lemma}\t{form}\t{feats}\n")


if __name__ == "__main__":
    main(sys.argv[1])
