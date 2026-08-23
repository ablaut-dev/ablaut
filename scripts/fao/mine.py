#!/usr/bin/env python3
"""Faroese productive weak-verb rules + mined exceptions.

Faroese is a Germanic language close to Icelandic, cited by its infinitive
(overwhelmingly in -a). Generated productively by rule (see produce(),
mirrored in src/fao.rs) is the regular class-1 weak (-a / -aði) paradigm:

  stem  = infinitive minus final -a          (kasta -> kast)
  psg   = present-singular stem; -ja drops j  (síggja -> sígg)

  V;NFIN            = cit                 kasta
  V;IND;PRS;1;SG    = psg + i             kasti
  V;IND;PRS;2;SG    = stem + ar           kastar
  V;IND;PRS;3;SG    = stem + ar           kastar
  V;IND;PRS;3 (pl)  = cit                 kasta
  V;IND;PST;1..3;SG = stem + aði          kastaði
  V;IND;PST;3 (pl)  = stem + aðu          kastaðu
  V;IMP;2;SG        = psg                 kasta / bare stem
  V;IMP;2;PL        = stem + ið           kastið
  V.CVB (supine)    = stem + að           kastað
  V.PTCP.PRS        = stem + andi         kastandi
  V.PTCP.PST        = stem + aður         kastaður

Everything that deviates — the strong verbs (ablaut past, -in participle),
the other weak classes (-di/-ti/-dur pasts, -ir/-ur present singulars), and
every stem-changing form — is lexical and mined into data/fao/parts.tsv.
Where the two oracles (UniMorph and kaikki.org) attest the same form we store
that; otherwise the UniMorph reading.

Usage: mine.py data/fao/unimorph.tsv data/fao/kaikki.tsv > data/fao/parts.tsv
"""
import sys
from collections import defaultdict

# Column order of parts.tsv: the scored synthetic cells.
CELLS = [
    "V;NFIN",
    "V;IND;PRS;1;SG", "V;IND;PRS;2;SG", "V;IND;PRS;3;SG", "V;IND;PRS;3",
    "V;IND;PST;1;SG", "V;IND;PST;2;SG", "V;IND;PST;3;SG", "V;IND;PST;3",
    "V;IMP;2;SG", "V;IMP;2;PL",
    "V.CVB", "V.PTCP.PRS", "V.PTCP.PST",
]


def produce(cit, feat):
    """The productive class-1 weak form, or None for a non -a citation."""
    if not cit.endswith("a"):
        return None
    stem = cit[:-1]
    psg = stem[:-1] if stem.endswith("j") else stem
    return {
        "V;NFIN": cit,
        "V;IND;PRS;1;SG": psg + "i",
        "V;IND;PRS;2;SG": stem + "ar",
        "V;IND;PRS;3;SG": stem + "ar",
        "V;IND;PRS;3": cit,
        "V;IND;PST;1;SG": stem + "aði",
        "V;IND;PST;2;SG": stem + "aði",
        "V;IND;PST;3;SG": stem + "aði",
        "V;IND;PST;3": stem + "aðu",
        "V;IMP;2;SG": psg,
        "V;IMP;2;PL": stem + "ið",
        "V.CVB": stem + "að",
        "V.PTCP.PRS": stem + "andi",
        "V.PTCP.PST": stem + "aður",
    }.get(feat)


def main(uni_path, kai_path):
    def read(path):
        d = defaultdict(set)
        for line in open(path):
            a = line.rstrip("\n").split("\t")
            if len(a) >= 3 and a[2].startswith("V"):
                d[(a[0], a[2])].add(a[1])
        return d
    uni, kai = read(uni_path), read(kai_path)

    def chosen(lemma, feat):
        a, b = uni.get((lemma, feat), set()), kai.get((lemma, feat), set())
        pool = (a & b) or a or b
        return sorted(pool)[0] if pool else None

    lemmas = sorted({l for (l, f) in list(uni) + list(kai) if f == "V;NFIN"})

    total = hit = 0
    parts = {}
    for lemma in lemmas:
        if not lemma or " " in lemma:
            continue
        row = {}
        for c in CELLS:
            gold = chosen(lemma, c)
            if gold is None:
                continue
            pred = produce(lemma, c)
            if pred is not None:
                total += 1
                if pred == gold:
                    hit += 1
            if pred != gold:
                row[c] = gold
        if row:
            parts[lemma] = row

    sys.stderr.write(
        f"rule accuracy: {hit}/{total} = {100 * hit / max(total, 1):.2f}%, "
        f"mined lemmas: {len(parts)}\n"
    )
    print("# lemma(infinitive)\t" + "\t".join(CELLS) + '  ("-" = productive default)')
    for lemma in sorted(parts):
        print(lemma + "\t" + "\t".join(parts[lemma].get(c, "-") for c in CELLS))


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
