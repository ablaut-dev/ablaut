#!/usr/bin/env python3
"""Luxembourgish productive present/imperative/participle rules + mined
exceptions. Validates rule accuracy against the single kaikki oracle and
emits data/ltz/parts.tsv (only the forms that deviate from the rules).

The regular Luxembourgish verb is built from a stem = infinitive minus the
`-en` ending (schaffen -> schaff-):

  V;NFIN            = infinitive              schaffen
  V;IND;PRS;SG;1    = infinitive              schaffen
  V;IND;PRS;SG;2    = stem + s                schaffs
  V;IND;PRS;SG;3    = stem + t                schafft
  V;IND;PRS;PL;1    = infinitive              schaffen
  V;IND;PRS;PL;2    = stem + t                schafft
  V;IND;PRS;PL;3    = infinitive              schaffen
  V;IMP;2;SG        = stem                    schaff
  V;IMP;2;PL        = stem + t                schafft
  V.PTCP;PST        = ge + stem + t           geschafft

Two orthographic adjustments generalise across the paradigm:
  * a stem-final `t` swallows the added `t` (retten -> rett, not *rettt);
  * a stem-final sibilant `s`/`z` swallows the 2sg `s` (weisen -> weis,
    setzen -> setz).
The past participle drops `ge-` on the unstressed inseparable prefixes
(be-, ver-, er-, zer-, ent-, emp-, miss-, ge-) and on the `-éieren` loan
class (studéieren -> studéiert). Strong participles, ablaut/umlaut present
stems, the separable-prefix `ge-` infix and the suppletive auxiliaries all
deviate and are mined verbatim.

Usage: mine.py data/ltz/kaikki.tsv > data/ltz/parts.tsv"""
import sys
from collections import defaultdict

CELLS = [
    "V;NFIN",
    "V;IND;PRS;SG;1", "V;IND;PRS;SG;2", "V;IND;PRS;SG;3",
    "V;IND;PRS;PL;1", "V;IND;PRS;PL;2", "V;IND;PRS;PL;3",
    "V;IMP;2;SG", "V;IMP;2;PL",
    "V.PTCP;PST",
]

INSEP = ("be", "ver", "er", "zer", "ent", "emp", "miss", "ge")


def plus_t(stem):
    """Append the dental suffix, collapsing a doubled t (rett + t -> rett)."""
    return stem if stem.endswith("t") else stem + "t"


def produce(cit, feat):
    # The infinitive ends in -en, or -ën in the vowel-final `-eën` class
    # (dreeën, beleeën); the stem is the citation minus that ending.
    if not (cit.endswith("en") or cit.endswith("ën")):
        return None
    stem = cit[:-2]
    if feat in ("V;NFIN", "V;IND;PRS;SG;1", "V;IND;PRS;PL;1", "V;IND;PRS;PL;3"):
        return cit
    if feat == "V;IND;PRS;SG;2":
        return stem if stem.endswith(("s", "z")) else stem + "s"
    if feat == "V;IND;PRS;SG;3":
        return plus_t(stem)
    if feat == "V;IND;PRS;PL;2":
        return plus_t(stem)
    if feat == "V;IMP;2;SG":
        return stem
    if feat == "V;IMP;2;PL":
        return plus_t(stem)
    if feat == "V.PTCP;PST":
        noge = cit.endswith("éieren") or any(cit.startswith(p) for p in INSEP)
        return plus_t(stem if noge else "ge" + stem)
    return None


def main(path):
    gold = defaultdict(dict)
    for l in open(path):
        a = l.rstrip("\n").split("\t")
        if len(a) >= 3:
            gold[a[0]].setdefault(a[2], set()).add(a[1])
    total = hit = 0
    parts = {}
    for cit, cells in gold.items():
        regular = cit.endswith("en") or cit.endswith("ën")
        row = {}
        for feat, forms in cells.items():
            if feat not in CELLS:
                continue
            if not regular:
                # Wholly irregular citation (sinn, ginn, hunn, gesinn …):
                # store every attested cell verbatim so it conjugates.
                row[feat] = sorted(forms)[0]
                continue
            pred = produce(cit, feat)
            if pred is None:
                continue
            total += 1
            if pred in forms:
                hit += 1
            else:
                # store the first attested variant as the mined form
                row[feat] = sorted(forms)[0]
        if row:
            parts[cit] = row
    sys.stderr.write(
        f"rule accuracy: {hit}/{total} = {100*hit/max(total,1):.2f}%, "
        f"mined lemmas: {len(parts)}\n"
    )
    print("# lemma(infinitive)\t" + "\t".join(CELLS))
    for cit in sorted(parts):
        print(cit + "\t" + "\t".join(parts[cit].get(c, "-") for c in CELLS))


if __name__ == "__main__":
    main(sys.argv[1])
