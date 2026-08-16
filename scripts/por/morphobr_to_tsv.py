#!/usr/bin/env python3
"""Convert the MorphoBr verb dictionaries (form<TAB>lemma+V+TAG+P+N)
into the lemma<TAB>form<TAB>features TSV the Portuguese golden harness
reads.

Usage: python3 scripts/por/morphobr_to_tsv.py data/por/morphobr/*.dict \
           > data/por/morphobr.tsv

MorphoBr tags → shared schema: PRS present, IMPF imperfect, PRF
preterite, PQP synthetic pluperfect, FUT future, COND conditional,
SBJR/SBJP/SBJF present/imperfect/future subjunctive, IMP imperative,
INF personal infinitive (bare INF = impersonal), GRD gerund, PTPST
participle (+F/M+SG/PL).
"""
import sys

TENSE = {
    "PRS": "V;IND;PRS",
    "IMPF": "V;IND;PST;IPFV",
    "PRF": "V;IND;PST;PFV",
    "PQP": "V;IND;PST;PQP",
    "FUT": "V;IND;FUT",
    "COND": "V;COND",
    "SBJR": "V;SBJV;PRS",
    "SBJP": "V;SBJV;PST",
    "SBJF": "V;SBJV;FUT",
    "IMP": "V;IMP",
    "INF": "V;NFIN",
}
GENDER = {"F": "FEM", "M": "MASC"}
NUMBER = {"SG": "SG", "PL": "PL"}


def features(parts):
    """parts = tag tokens after +V+."""
    if not parts:
        return None
    tag = parts[0]
    if tag == "GRD":
        return "V;GER"
    if tag == "PTPST":
        if len(parts) == 3 and parts[1] in GENDER and parts[2] in NUMBER:
            return f"V.PTCP;PST;{GENDER[parts[1]]};{NUMBER[parts[2]]}"
        return None
    if tag == "INF" and len(parts) == 1:
        return "V;NFIN"
    key = TENSE.get(tag)
    if key and len(parts) == 3 and parts[1] in "123" and parts[2] in NUMBER:
        return f"{key};{NUMBER[parts[2]]};{parts[1]}"
    return None


def main(paths):
    seen = set()
    for path in paths:
        for line in open(path):
            line = line.rstrip("\n")
            if "\t" not in line:
                continue
            form, analysis = line.split("\t", 1)
            bits = analysis.split("+")
            if len(bits) < 2 or bits[1] != "V":
                continue
            lemma = bits[0]
            feat = features(bits[2:])
            if feat is None:
                continue
            row = (lemma, form, feat)
            if row not in seen:
                seen.add(row)
                sys.stdout.write(f"{lemma}\t{form}\t{feat}\n")


if __name__ == "__main__":
    main(sys.argv[1:])
