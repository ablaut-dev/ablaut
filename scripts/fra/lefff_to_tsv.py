#!/usr/bin/env python3
"""Convert the Lefff 3.4 extensional lexicon (.mlex) into the
lemma<TAB>form<TAB>features TSV that the French golden harness reads.

Usage: python3 scripts/fra/lefff_to_tsv.py data/fra/lefff-3.4.mlex > data/fra/lefff.tsv

The .mlex format is form<TAB>pos<TAB>lemma<TAB>morphtag, where a verb morphtag
is a compact code: tense letters, then persons, then number, e.g. "PS13s" =
{present indicative, present subjunctive} x {1st, 3rd} x singular.

Tense letters: P present ind, I imparfait, J passe simple, F future,
C conditional, Y imperative, S subjunctive present, T subjunctive imperfect,
W infinitive, G present participle, K past participle (+m/f +s/p).
"""
import sys

TENSE = {
    "P": "V;IND;PRS",
    "I": "V;IND;PST;IPFV",
    "J": "V;IND;PST;PFV",
    "F": "V;IND;FUT",
    "C": "V;COND",
    "Y": "V;IMP",
    "S": "V;SBJV;PRS",
    "T": "V;SBJV;PST",
}
GENDER = {"m": "MASC", "f": "FEM"}
NUMBER = {"s": "SG", "p": "PL"}


def features(tag):
    """Expand a Lefff verb morphtag into a list of feature strings."""
    if tag == "W":
        return ["V;NFIN"]
    if tag.startswith("G"):
        return ["V.PTCP;PRS"]
    if tag.startswith("K"):
        genders = [GENDER[c] for c in tag if c in GENDER] or ["MASC"]
        numbers = [NUMBER[c] for c in tag if c in NUMBER] or ["SG"]
        return [f"V.PTCP;PST;{g};{n}" for g in genders for n in numbers]
    tenses = [TENSE[c] for c in tag if c in TENSE]
    persons = [c for c in tag if c in "123"]
    numbers = [NUMBER[c] for c in tag if c in NUMBER]
    if not (tenses and persons and numbers):
        return []
    return [
        f"{t};{n};{p}" for t in tenses for n in numbers for p in persons
    ]


def main(path):
    seen = set()
    for line in open(path):
        parts = line.rstrip("\n").split("\t")
        if len(parts) != 4 or parts[1] != "v":
            continue
        form, _, lemma, tag = parts
        form, lemma = form.strip(), lemma.strip()
        # Symbol pseudo-verbs (">" for "suivre" etc.) and multiword entries
        # are lexicographic artifacts, not conjugation.
        if not form.isalpha() and "'" not in form and "-" not in form:
            continue
        for feat in features(tag):
            row = (lemma, form, feat)
            if row not in seen:
                seen.add(row)
                sys.stdout.write(f"{lemma}\t{form}\t{feat}\n")


if __name__ == "__main__":
    main(sys.argv[1])
