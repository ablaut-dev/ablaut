#!/usr/bin/env python3
"""Convert the BÍN (Beygingarlýsing íslensks nútímamáls, Árni Magnússon
Institute) Sigrúnarsnið CSV into the lemma<TAB>form<TAB>features TSV the
Icelandic golden harness reads.

Usage: python3 scripts/isl/bin_to_tsv.py data/isl/SHsnid.csv > data/isl/bin.tsv

BÍN rows are semicolon-separated: lemma;id;wordclass;domain;form;tag.
Verbs are wordclass `so`. The grammatical tag is voice-mood-tense-
person-number, e.g. GM-FH-NT-1P-ET:
  voice  GM active / MM middle (we keep active only)
  mood   FH indicative / VH subjunctive / BH imperative / NH infinitive
  tense  NT present / ÞT past
  person 1P/2P/3P    number ET singular / FT plural
Non-finite: GM-NH infinitive, LHNT present participle, GM-SAGNB supine
(the indeclinable past participle used with hafa — our single V.PTCP;PST).
The declined past participle (LHÞT-...) and the question/clitic forms
(SP-...) are out of the conjugator's schema and skipped.

The feature schema matches scripts/isl/kaikki_to_tsv.py.
"""
import sys

NUM = {"ET": "SG", "FT": "PL"}


def features(tag):
    """Map a BÍN active-voice verb tag to a feature string, or None."""
    if tag == "GM-NH":
        return "V;NFIN"
    if tag == "LHNT":
        return "V;V.PTCP;PRS"
    if tag == "GM-SAGNB":
        return "V.PTCP;PST"
    # Imperative: BH-ST is the clipped 2sg stem (far), BH-FT the 2pl
    # (farið); BH-ET is the enclitic-pronoun form (farðu) and is skipped.
    if tag == "GM-BH-ST":
        return "V;IMP;SG;2"
    if tag == "GM-BH-FT":
        return "V;IMP;PL;2"
    parts = tag.split("-")
    if len(parts) != 5:
        return None
    voice, mood, tense, person, number = parts
    if voice != "GM":
        return None
    n = NUM.get(number)
    if n is None or person not in ("1P", "2P", "3P"):
        return None
    p = person[0]
    if mood == "FH":
        base = "V;IND"
    elif mood == "VH":
        base = "V;SBJV"
    else:
        return None
    if tense == "NT":
        t = "PRS"
    elif tense == "ÞT":
        t = "PST"
    else:
        return None
    return f"{base};{t};{n};{p}"


def main(path):
    seen = set()
    for line in open(path, encoding="utf-8"):
        f = line.rstrip("\n").split(";")
        if len(f) < 6 or f[2] != "so":
            continue
        lemma, form, tag = f[0], f[4], f[5]
        if not form or form == "-":
            continue
        feat = features(tag)
        if feat is None:
            continue
        row = (lemma, form, feat)
        if row not in seen:
            seen.add(row)
            sys.stdout.write(f"{lemma}\t{form}\t{feat}\n")


if __name__ == "__main__":
    main(sys.argv[1])
