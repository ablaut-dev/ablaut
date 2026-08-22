#!/usr/bin/env python3
"""Convert UniMorph tel to the shared `lemma ⇥ form ⇥ features` TSV.

UniMorph tel (Batsuren & Cotterell) is an English-Wiktionary extraction:
116 verb lemmas, tagged with person, number and — in the third person —
gender, across three tenses (past, present-durative `PRS;DUR`, future).
Its bundles are already the schema the harness uses, so the only work
here is to keep the verb rows and drop duplicates.

Usage: python3 scripts/tel/unimorph_to_tsv.py data/tel/unimorph-tel.txt
"""

import sys


def main(path):
    rows = set()
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            fields = line.rstrip("\n").split("\t")
            if len(fields) != 3:
                continue
            lemma, form, features = (f.strip() for f in fields)
            if not features.startswith("V"):
                continue
            if not lemma or not form:
                continue
            rows.add((lemma, form, features))
    out = sys.stdout
    for lemma, form, features in sorted(rows):
        out.write(f"{lemma}\t{form}\t{features}\n")


if __name__ == "__main__":
    main(sys.argv[1])
