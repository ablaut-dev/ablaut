#!/usr/bin/env python3
"""Convert UniMorph hye to the shared `lemma ⇥ form ⇥ features` TSV.

UniMorph hye is an English-Wiktionary extraction (Batsuren & Cotterell,
validated by Hossep Dolatian): 898 verb lemmas × ~150 slots, including
the analytic tenses spelled out as multi-word forms (`կիսում եմ`,
`չեմ կիսել`).  Its feature bundles are already the schema the harness
uses, so the only work here is to drop the nominal half of the dump and
normalize the imperative's emphasis mark.

Usage: python3 scripts/hye/unimorph_to_tsv.py data/hye/unimorph-hye.txt
"""

import sys

# ARMENIAN EMPHASIS MARK: Wiktionary's imperatives carry it (գրի՛ր), the
# grammar-derived oracle does not. It is a prosodic diacritic, not part
# of the spelling of the form.
EMPHASIS = "՛"


def normalize(text):
    """Reformed-orthography spelling of a lemma or form.

    A handful of Wiktionary entries keep the pre-1922 digraph եւ where
    the reform writes the ligature և (անձրեւել → անձրևել). Reformed
    orthography has no other ե+ւ sequence — ւ survives only inside ու —
    so the rewrite is unambiguous.
    """
    return text.replace(EMPHASIS, "").replace("եւ", "և")


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
            lemma, form = normalize(lemma), normalize(form)
            if not form:
                continue
            rows.add((lemma, form, features))
    out = sys.stdout
    for lemma, form, features in sorted(rows):
        out.write(f"{lemma}\t{form}\t{features}\n")


if __name__ == "__main__":
    main(sys.argv[1])
