#!/usr/bin/env python3
"""Convert the SGJP (Grammatical Dictionary of Polish) tab dump to the
shared TSV, under the same UniMorph-pol feature strings so the harness can
intersect the two into agreement gold.

SGJP rows are `form <TAB> lemma <TAB> tag` where tag is colon-separated
(e.g. fin:sg:pri:imperf, praet:sg:m1:imperf, impt:pl:sec, inf). We map the
synthetic finite paradigm: the present (imperfective fin) / synthetic
future (perfective fin), the gendered past l-form (praet, the person-clitic
forms are UniMorph-only), the imperative and the infinitive. Negated
(neg) forms and the heavily case-inflected participles are skipped.
"""
import sys

PERSON = {"pri": "1", "sec": "2", "ter": "3"}
# SGJP gender tags -> UniMorph past cell suffix.
GENDER_SG = {"m1": "MASC", "m2": "MASC", "m3": "MASC", "f": "FEM", "n": "NEUT"}


def feature(tag):
    parts = tag.split(":")
    p = set(parts)
    if "neg" in p or "nonpast" in p:
        return None
    pos = parts[0]
    number = "SG" if "sg" in p else "PL" if "pl" in p else None
    person = next((v for k, v in PERSON.items() if k in p), None)

    if pos == "inf":
        return "V;NFIN"
    if pos == "fin" and person and number:
        # imperfective fin = present; perfective fin = synthetic future.
        if "imperf" in p:
            return f"V;PRS;{person};{number}"
        if "perf" in p:
            return f"V;FUT;{person};{number}"
        return None
    if pos == "impt" and person and number:
        return f"V;IMP;{person};{number}"
    if pos == "praet" and number:
        # The clitic-less l-form is UniMorph's 3rd person; gendered.
        if number == "SG":
            g = next((GENDER_SG[k] for k in parts if k in GENDER_SG), None)
            return f"V;PST;3;SG;{g}" if g else None
        # Plural: virile (m1) = MASC;HUM, else the bare plural.
        return "V;PST;3;PL;MASC;HUM" if "m1" in p else "V;PST;3;PL"
    return None


def main(path):
    import gzip
    op = gzip.open if path.endswith(".gz") else open
    for line in op(path, "rt", encoding="utf-8"):
        if line.startswith("#"):
            continue
        cols = line.rstrip("\n").split("\t")
        if len(cols) < 3:
            continue
        form, lemma, tag = cols[0], cols[1], cols[2]
        if not form or not lemma or " " in form or " " in lemma:
            continue
        feat = feature(tag)
        if feat:
            print(f"{lemma}\t{form}\t{feat}")


if __name__ == "__main__":
    main(sys.argv[1])
