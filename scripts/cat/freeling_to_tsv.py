#!/usr/bin/env python3
"""Convert the FreeLing Catalan verb entries (EAGLES tags) into the
lemma<TAB>form<TAB>features TSV that the Catalan golden harness reads.

Usage: python3 scripts/cat/freeling_to_tsv.py data/cat/freeling-verbs.src \
           > data/cat/freeling.tsv

An EAGLES verb tag is 7 characters: V, category (M main / A aux /
S semiauxiliary), mood (I indicative, S subjunctive, M imperative,
N infinitive, G gerund, P participle), tense (P present, I imperfect,
F future, S past, C conditional, 0), person, number, gender.

The feature schema matches scripts/cat/kaikki_to_tsv.py so
cross_oracle.py can compare the two directly. Both imperfect
subjunctives (-ra and -se) carry the same EAGLES tag and so become
variants of one V;SBJV;PST slot.
"""
import sys

TENSE = {
    ("I", "P"): "V;IND;PRS",
    ("I", "I"): "V;IND;PST;IPFV",
    ("I", "S"): "V;IND;PST;PFV",
    ("I", "F"): "V;IND;FUT",
    ("I", "C"): "V;COND",
    ("S", "P"): "V;SBJV;PRS",
    ("S", "I"): "V;SBJV;PST",
    ("S", "F"): "V;SBJV;FUT",
}
NUMBER = {"S": "SG", "P": "PL"}
GENDER = {"M": "MASC", "F": "FEM"}


def features(tag):
    """Expand an EAGLES verb tag into a feature string, or None."""
    if len(tag) != 7 or tag[0] != "V":
        return None
    mood, tense, person, number, gender = tag[2], tag[3], tag[4], tag[5], tag[6]
    if mood == "N":
        return "V;NFIN"
    if mood == "G":
        return "V;GER"
    if mood == "P":
        n = NUMBER.get(number)
        g = GENDER.get(gender)
        if n and g:
            return f"V.PTCP;PST;{g};{n}"
        return None
    if mood == "M":
        n = NUMBER.get(number)
        if n and person in "123":
            return f"V;IMP;{n};{person}"
        return None
    key = TENSE.get((mood, tense))
    n = NUMBER.get(number)
    if key and n and person in "123":
        return f"{key};{n};{person}"
    return None


def main(path):
    seen = set()
    for line in open(path):
        parts = line.split()
        if len(parts) != 3:
            continue
        form, lemma, tag = parts
        feat = features(tag)
        if feat is None:
            continue
        row = (lemma, form, feat)
        if row not in seen:
            seen.add(row)
            sys.stdout.write(f"{lemma}\t{form}\t{feat}\n")


if __name__ == "__main__":
    main(sys.argv[1])
