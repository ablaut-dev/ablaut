#!/usr/bin/env python3
"""Yiddish productive present/participle rules + mined exceptions.

Validates rule accuracy against kaikki and emits data/ydd/parts.tsv (only the
cells that deviate from the productive rules — chiefly the strong verbs, whose
past participle carries lexical ablaut). src/ydd.rs mirrors produce() exactly.

Usage: mine.py data/ydd/kaikki.tsv > data/ydd/parts.tsv
"""
import sys
from collections import defaultdict

RAFE = "ֿ"  # combining rafe, sits on פֿ etc.


def ispoint(c):
    return 0x591 <= ord(c) <= 0x5c7


# Final-form <-> non-final-form of the five Yiddish consonants that have one.
FIN = {"כ": "ך", "מ": "ם", "נ": "ן", "פ": "ף", "צ": "ץ"}
DEFIN = {"ך": "כ", "ם": "מ", "ן": "נ", "ף": "פ", "ץ": "צ"}
# Sonorants after which infinitival -ען is a syllabic ending that the stem drops.
SYLL = set("למנגײ")
# Unstressed prefixes: their verbs take no ge- in the past participle.
UNSTR = ["באַ", "גע", "דער", "פֿאַר", "פֿער", "צע", "אַנט", "ער", "מיס"]
# Separable (stressed) prefixes: ge- is infixed after them (prefix-ge-stem-t).
SEP = sorted(
    ["אַרויס", "אַרײַן", "אַראָפּ", "אַרום", "אַנידער", "אַוועק", "אונטער", "איבער",
     "אויס", "אויפֿ", "אויף", "אָפּ", "אָן", "אײַן", "צונויפֿ", "צוזאַמען", "צוריק",
     "צו", "מיט", "נאָך", "פֿאָר", "פֿונאַנדער", "ווײַטער", "דורך", "אַהיים"],
    key=len, reverse=True,
)

CELLS = ["V;PRS;1;SG", "V;PRS;2;SG", "V;PRS;3;SG", "V;PRS;1;PL", "V;PRS;2;PL",
         "V;PRS;3;PL", "V;IMP;2;SG", "V;IMP;2;PL", "V.PTCP;PRS", "V.PTCP;PST",
         "V;NFIN"]


def lastbase(s):
    for c in reversed(s):
        if not ispoint(c):
            return c
    return ""


def finalize(s):
    """Word-final form: convert a trailing non-final consonant to its final form."""
    if not s:
        return s
    if s[-1] == RAFE:                      # פֿ -> ף (final fey, rafe dropped)
        if len(s) >= 2 and s[-2] == "פ":
            return s[:-2] + "ף"
        return s
    if s[-1] in FIN:
        return s[:-1] + FIN[s[-1]]
    return s


def definalize(s):
    """Word-internal form: undo a final consonant so a suffix can follow."""
    if s and s[-1] in DEFIN:
        return s[:-1] + DEFIN[s[-1]]
    return s


def stem_of(inf):
    """Present stem: drop -n, or syllabic -en after a sonorant."""
    if inf.endswith("ען"):
        return inf[:-2] if lastbase(inf[:-2]) in SYLL else inf[:-1]
    if inf.endswith("ן"):
        return inf[:-1]
    return inf


def produce(inf, feat):
    if feat == "V;NFIN":
        return inf
    if feat == "V.PTCP;PRS":
        return definalize(inf) + "דיק"
    if feat == "V.PTCP;PST":
        st = stem_of(inf)
        for pre in UNSTR:
            if inf.startswith(pre):
                return st + "ט"
        for pre in SEP:
            if inf.startswith(pre):
                return pre + "גע" + stem_of(inf[len(pre):]) + "ט"
        return "גע" + st + "ט"
    st = stem_of(inf)
    lb = lastbase(st)
    tf = lb in "טד"                        # t/d-final stem: -t merges
    if feat == "V;IMP;2;SG":
        return finalize(st)
    if feat == "V;IMP;2;PL":
        return st if tf else st + "ט"
    p = feat.split(";")
    pn = p[2] + ";" + p[3]
    if pn == "1;SG":
        return finalize(st)
    if pn in ("1;PL", "3;PL"):
        return inf
    if pn in ("3;SG", "2;PL"):
        return st if tf else st + "ט"
    if pn == "2;SG":
        return st + "ט" if lb == "ס" else st + "סט"   # -s + -st degeminates
    return None


def main(path):
    gold = defaultdict(dict)
    for l in open(path):
        a = l.rstrip("\n").split("\t")
        if len(a) >= 3:
            gold[a[0]].setdefault(a[2], set()).add(a[1])
    total = hit = 0
    parts = {}
    for inf, feats in gold.items():
        row = {}
        for feat, forms in feats.items():
            if feat not in CELLS:
                continue
            pred = produce(inf, feat)
            if pred is None:
                continue
            total += 1
            if pred in forms:
                hit += 1
            else:
                row[feat] = sorted(forms)[0]
        if row:
            parts[inf] = row
    sys.stderr.write(
        f"rule accuracy: {hit}/{total} = {100*hit/max(total,1):.2f}%, "
        f"mined lemmas: {len(parts)}\n"
    )
    print("# lemma(infinitive)\t" + "\t".join(CELLS))
    for inf in sorted(parts):
        print(inf + "\t" + "\t".join(parts[inf].get(c, "-") for c in CELLS))


if __name__ == "__main__":
    main(sys.argv[1])
