#!/usr/bin/env python3
"""Galician productive -ar/-er/-ir conjugation + mined exceptions.

Galician is a Romance language cited by its infinitive. The regular
paradigm is a clean three-class system keyed by the infinitive ending
(-ar / -er / -ir): from the stem (infinitive minus the two-letter ending)
a fixed table of endings produces every synthetic cell — the ten
person/number tense blocks, the imperative, the gerund and the four
participle forms. `produce()` below is mirrored EXACTLY by
`productive()` in src/glg.rs.

Everything a rule gets wrong — irregular verbs (ser, ir, ter, facer,
dicir, poñer, …), orthographic stem changes (-cer/-cir c→z, -gar/-car in
the preterite and present subjunctive, -guer/-guir gu→g), and the like —
is mined into data/glg/parts.tsv, one column per cell. Single oracle
(kaikki.org): Beta.

Usage: mine.py data/glg/kaikki.tsv > data/glg/parts.tsv"""
import sys
from collections import defaultdict

# Column order of parts.tsv, mirrored by CELLS in src/glg.rs.
NONFIN = ["V;NFIN", "V;GER", "V;PTCP;MASC;SG", "V;PTCP;FEM;SG",
          "V;PTCP;MASC;PL", "V;PTCP;FEM;PL"]
TENSES = ["V;IND;PRS", "V;IND;IPFV", "V;IND;PRET", "V;IND;PLUP",
          "V;IND;FUT", "V;COND", "V;SBJV;PRS", "V;SBJV;PST",
          "V;SBJV;FUT", "V;INF"]
PN = [(p, n) for p in "123" for n in ("SG", "PL")]
CELLS = NONFIN + [f"{t};{p};{n}" for t in TENSES for (p, n) in PN] \
        + ["V;IMP;2;SG", "V;IMP;2;PL"]

# Per-class ending tables. The key of the person/number tenses is
# "TENSE p n"; non-finite cells key by their bare feature. The ending is
# appended to the stem (infinitive minus the two-letter class suffix).
_PN = ["1 SG", "2 SG", "3 SG", "1 PL", "2 PL", "3 PL"]


def _block(base, ends):
    return {f"{base} {pn}": e for pn, e in zip(_PN, ends)}


def endings(cls):
    e = {
        "V;NFIN": cls,
        "V;GER": {"ar": "ando", "er": "endo", "ir": "indo"}[cls],
        "V;PTCP;MASC;SG": {"ar": "ado", "er": "ido", "ir": "ido"}[cls],
        "V;PTCP;FEM;SG": {"ar": "ada", "er": "ida", "ir": "ida"}[cls],
        "V;PTCP;MASC;PL": {"ar": "ados", "er": "idos", "ir": "idos"}[cls],
        "V;PTCP;FEM;PL": {"ar": "adas", "er": "idas", "ir": "idas"}[cls],
        "V;IMP;2;SG": {"ar": "a", "er": "e", "ir": "e"}[cls],
        "V;IMP;2;PL": {"ar": "ade", "er": "ede", "ir": "ide"}[cls],
    }
    if cls == "ar":
        e.update(_block("V;IND;PRS", ["o", "as", "a", "amos", "ades", "an"]))
        e.update(_block("V;IND;IPFV", ["aba", "abas", "aba", "abamos", "abades", "aban"]))
        e.update(_block("V;IND;PRET", ["ei", "aches", "ou", "amos", "astes", "aron"]))
        e.update(_block("V;IND;PLUP", ["ara", "aras", "ara", "aramos", "arades", "aran"]))
        e.update(_block("V;IND;FUT", ["arei", "arás", "ará", "aremos", "aredes", "arán"]))
        e.update(_block("V;COND", ["aría", "arías", "aría", "ariamos", "ariades", "arían"]))
        e.update(_block("V;SBJV;PRS", ["e", "es", "e", "emos", "edes", "en"]))
        e.update(_block("V;SBJV;PST", ["ase", "ases", "ase", "ásemos", "ásedes", "asen"]))
        e.update(_block("V;SBJV;FUT", ["ar", "ares", "ar", "armos", "ardes", "aren"]))
        e.update(_block("V;INF", ["ar", "ares", "ar", "armos", "ardes", "aren"]))
    elif cls == "er":
        e.update(_block("V;IND;PRS", ["o", "es", "e", "emos", "edes", "en"]))
        e.update(_block("V;IND;IPFV", ["ía", "ías", "ía", "iamos", "iades", "ían"]))
        e.update(_block("V;IND;PRET", ["ín", "iches", "eu", "emos", "estes", "eron"]))
        e.update(_block("V;IND;PLUP", ["era", "eras", "era", "eramos", "erades", "eran"]))
        e.update(_block("V;IND;FUT", ["erei", "erás", "erá", "eremos", "eredes", "erán"]))
        e.update(_block("V;COND", ["ería", "erías", "ería", "eriamos", "eriades", "erían"]))
        e.update(_block("V;SBJV;PRS", ["a", "as", "a", "amos", "ades", "an"]))
        e.update(_block("V;SBJV;PST", ["ese", "eses", "ese", "ésemos", "ésedes", "esen"]))
        e.update(_block("V;SBJV;FUT", ["er", "eres", "er", "ermos", "erdes", "eren"]))
        e.update(_block("V;INF", ["er", "eres", "er", "ermos", "erdes", "eren"]))
    else:  # ir
        e.update(_block("V;IND;PRS", ["o", "es", "e", "imos", "ides", "en"]))
        e.update(_block("V;IND;IPFV", ["ía", "ías", "ía", "iamos", "iades", "ían"]))
        e.update(_block("V;IND;PRET", ["ín", "iches", "iu", "imos", "istes", "iron"]))
        e.update(_block("V;IND;PLUP", ["ira", "iras", "ira", "iramos", "irades", "iran"]))
        e.update(_block("V;IND;FUT", ["irei", "irás", "irá", "iremos", "iredes", "irán"]))
        e.update(_block("V;COND", ["iría", "irías", "iría", "iriamos", "iriades", "irían"]))
        e.update(_block("V;SBJV;PRS", ["a", "as", "a", "amos", "ades", "an"]))
        e.update(_block("V;SBJV;PST", ["ise", "ises", "ise", "ísemos", "ísedes", "isen"]))
        e.update(_block("V;SBJV;FUT", ["ir", "ires", "ir", "irmos", "irdes", "iren"]))
        e.update(_block("V;INF", ["ir", "ires", "ir", "irmos", "irdes", "iren"]))
    return e


ENDINGS = {c: endings(c) for c in ("ar", "er", "ir")}


def key(feat):
    """Map a CELLS feature to its ending-table key."""
    if feat in NONFIN or feat.startswith("V;IMP"):
        return feat
    parts = feat.split(";")  # V IND PRS 1 SG  ->  "V;IND;PRS 1 SG"
    tense = ";".join(parts[:-2])
    return f"{tense} {parts[-2]} {parts[-1]}"


def produce(cit, feat):
    cls = cit[-2:]
    if cls not in ENDINGS or len(cit) < 3:
        return None
    stem = cit[:-2]
    end = ENDINGS[cls].get(key(feat))
    return None if end is None else stem + end


def main(path):
    gold = defaultdict(lambda: defaultdict(set))
    for line in open(path):
        a = line.rstrip("\n").split("\t")
        if len(a) >= 3:
            gold[a[0]][a[2]].add(a[1])
    total = hit = 0
    parts = {}
    for cit in sorted(gold):
        if not cit or " " in cit:
            continue
        row = {}
        for feat in CELLS:
            forms = gold[cit].get(feat)
            if not forms:
                continue
            pred = produce(cit, feat)
            if pred is None:
                continue
            total += 1
            if pred in forms:
                hit += 1
            else:
                row[feat] = sorted(forms)[0]
        if row:
            parts[cit] = row
    sys.stderr.write(
        f"rule accuracy: {hit}/{total} = {100 * hit / max(total, 1):.2f}%, "
        f"mined lemmas: {len(parts)}/{len(gold)}\n")
    print("# lemma(infinitive)\t" + "\t".join(CELLS) + '  ("-" = productive default)')
    for cit in sorted(parts):
        print(cit + "\t" + "\t".join(parts[cit].get(c, "-") for c in CELLS))


if __name__ == "__main__":
    main(sys.argv[1])
