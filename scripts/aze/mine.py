#!/usr/bin/env python3
"""Azerbaijani productive vowel-harmony rules + mined exceptions.
Validates rule accuracy against kaikki and emits data/aze/parts.tsv
(only the forms that deviate from the rules).
Usage: mine.py data/aze/kaikki.tsv > data/aze/parts.tsv"""
import sys
from collections import defaultdict

VOWELS = "aıoueəiöü"
BACK = "aıou"
ROUND = "ouöü"

def harmony(stem):
    v = next((c for c in reversed(stem) if c in VOWELS), "ə")
    back = v in BACK
    rnd = v in ROUND
    A = "a" if back else "ə"
    I = ("u" if rnd else "ı") if back else ("ü" if rnd else "i")
    K = "q" if back else "k"
    return A, I, K

COP = None  # copular person, filled per stem
CELLS = ["V;PRS;1;SG","V;PRS;2;SG","V;PRS;3;SG","V;PRS;1;PL","V;PRS;2;PL","V;PRS;3;PL",
    "V;PST;1;SG","V;PST;2;SG","V;PST;3;SG","V;PST;1;PL","V;PST;2;PL","V;PST;3;PL",
    "V;FUT;1;SG","V;FUT;2;SG","V;FUT;3;SG","V;FUT;1;PL","V;FUT;2;PL","V;FUT;3;PL",
    "V;AOR;1;SG","V;AOR;2;SG","V;AOR;3;SG","V;AOR;1;PL","V;AOR;2;PL","V;AOR;3;PL",
    "V;IMP;2;SG","V;IMP;2;PL","V;NFIN"]

def produce(cit, feat):
    if not (cit.endswith("maq") or cit.endswith("mək")):
        return None
    if feat == "V;NFIN":
        return cit
    stem = cit[:-3]
    A, I, K = harmony(stem)
    y = "y" if stem and stem[-1] in VOWELS else ""
    cop = {"1;SG": A+"m", "2;SG": "s"+A+"n", "3;SG": "", "1;PL": I+K,
           "2;PL": "s"+I+"n"+I+"z", "3;PL": "l"+A+"r"}
    past = {"1;SG": "m", "2;SG": "n", "3;SG": "", "1;PL": K,
            "2;PL": "n"+I+"z", "3;PL": "l"+A+"r"}
    parts = feat.split(";")
    if feat == "V;IMP;2;SG": return stem
    if feat == "V;IMP;2;PL": return stem + y + I + "n"
    pn = parts[2] + ";" + parts[3]
    tense = parts[1]
    if tense == "PRS":
        return stem + y + I + "r" + cop[pn]
    if tense == "PST":
        return stem + "d" + I + past[pn]
    if tense == "AOR":
        return stem + y + A + "r" + cop[pn]
    if tense == "FUT":
        base = stem + y + A + "c" + A          # sevəcə / qadayaca
        p = cop[pn]
        if p and p[0] in VOWELS:               # vowel-initial: q→ğ, k→y
            return base + ("ğ" if K=="q" else "y") + p
        return base + K + p
    return None

def main(path):
    gold = defaultdict(dict)
    for l in open(path):
        a = l.rstrip("\n").split("\t")
        if len(a) >= 3:
            gold[a[0]].setdefault(a[2], set()).add(a[1])
    total=hit=0
    parts={}
    for cit, cells in gold.items():
        row = {}
        for feat, forms in cells.items():
            if feat not in CELLS: continue
            pred = produce(cit, feat)
            if pred is None: continue
            total += 1
            if pred in forms: hit += 1
            else:
                # store the first attested as the mined form
                row[feat] = sorted(forms)[0]
        if row: parts[cit] = row
    sys.stderr.write(f"rule accuracy: {hit}/{total} = {100*hit/max(total,1):.2f}%, mined lemmas: {len(parts)}\n")
    print("# lemma(infinitive)\t" + "\t".join(CELLS))
    for cit in sorted(parts):
        print(cit + "\t" + "\t".join(parts[cit].get(c,"-") for c in CELLS))

if __name__=="__main__": main(sys.argv[1])
