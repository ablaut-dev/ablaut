#!/usr/bin/env python3
"""Convert UniMorph kan to the shared `lemma ⇥ form ⇥ features` TSV.

UniMorph kan (Batsuren & Cotterell) is an English-Wiktionary extraction.
Its 41 verb lemmas carry a full literary paradigm: person, number and —
in the third person — a three-way gender (masculine / feminine /
neuter), across the past (`PST`), present (`PRS`) and future (`FUT`)
tenses, plus the imperative (`IMP`).

Two kinds of noise are dropped so the gold is clean:

* the negative, positive-emphatic and potential columns
  (`NEG`/`POS`/`POT`) and the tense-less bare-person bundles are filled,
  for every lemma, with a bare *pronoun* placeholder (ಅವನು "he",
  ನೀವು "you", …) rather than a verb form — Wiktionary's tables leave
  those cells to the pronoun. They are not verb forms, so they are
  excluded here (matched against the pronoun list, and by keeping only
  the tensed finite bundles + the imperative);
* the dubitative/"contingent" future doublet (ಮಾಡಿಯೇನು) is kept: it
  shares the `FUT` tag with the primary future in UniMorph.

Usage: python3 scripts/kan/unimorph_to_tsv.py data/kan/unimorph-kan.txt
"""

import sys

# Kannada personal pronouns Wiktionary leaves in the NEG/POS/POT cells.
PRONOUNS = {
    "ನಾನು", "ನೀನು", "ಅವನು", "ಅವಳು", "ಅದು",
    "ನಾವು", "ನೀವು", "ಅವರು", "ಅವು",
}
TENSES = {"PST", "PRS", "FUT"}


def keep(features, form):
    if form in PRONOUNS:
        return False
    if not features.startswith("V;"):
        return False  # drops V.PTCP participles and bare V
    toks = features.split(";")
    # Finite: a tensed bundle (…;PST/PRS/FUT) or the imperative (…;IMP).
    return toks[-1] in TENSES or toks[-1] == "IMP"


def main(path):
    rows = set()
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            fields = line.rstrip("\n").split("\t")
            if len(fields) != 3:
                continue
            lemma, form, features = (f.strip() for f in fields)
            if not lemma or not form:
                continue
            if keep(features, form):
                rows.add((lemma, form, features))
    out = sys.stdout
    for lemma, form, features in sorted(rows):
        out.write(f"{lemma}\t{form}\t{features}\n")


if __name__ == "__main__":
    main(sys.argv[1])
