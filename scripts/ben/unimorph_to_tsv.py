#!/usr/bin/env python3
"""Convert UniMorph ben to the shared `lemma ⇥ form ⇥ features` TSV.

UniMorph ben (Batsuren & Cotterell; Wikipedia/Wiktionary lineage) is a
generated verb paradigm: 84 verb lemmas, each cited by its `-আ` verbal
noun, with the full Bengali finite grid — five person × honorific
classes (আমি; তুই = LGSPEC1; তুমি = 3;INFM; সে = 2;POL; আপনি/তিনি =
3;POL) crossing eight tense-aspects (simple present/past, future, past
habitual, present/past progressive, present/past perfect) — plus the
non-finite forms (the `-তে` infinitive V.NFIN, the verbal noun V.MSDR,
and the perfective / habitual / progressive / conditional participles).

The file also carries nouns and adjectives (the whole `ben` UniMorph is
one file); only the `V` rows are kept. Bengali ben has no NEG/PASS rows
to drop, and — unlike Gujarati — candrabindu (ঁ, nasalization) and
anusvara (ং) are distinct phonemes here, so neither is folded.

Usage: python3 scripts/ben/unimorph_to_tsv.py data/ben/unimorph-ben.txt
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
            if not features.startswith("V") or not lemma or not form:
                continue
            rows.add((lemma, form, features))
    out = sys.stdout
    for lemma, form, features in sorted(rows):
        out.write(f"{lemma}\t{form}\t{features}\n")


if __name__ == "__main__":
    main(sys.argv[1])
