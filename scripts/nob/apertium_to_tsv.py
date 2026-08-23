#!/usr/bin/env python3
"""Convert `lt-expand` output of apertium-nob into the common golden TSV:
`lemma <TAB> form <TAB> features`.

apertium-nob is the independent oracle: a hand-built lttoolbox dictionary
with no Wiktionary lineage. `lt-expand` walks it into `surface:lemma<tags>`
pairs; this maps its verb tag set onto the UniMorph-style bundle the
engine and the UniMorph adapter share, so the harness can intersect them.

Tag mapping (apertium <vblex> → shared bundle):
  pres              → V;IND;PRS      (kaster)
  pret              → V;IND;PST      (kasta / kastet)
  pp                → V.PTCP;PST     (kastet)
  imp               → V;IMP          (kast)
  pres + pasv       → V;IND;PASS     (kastes)
The infinitive (inf), the passive infinitive (inf+pasv) and the
adjectival present participle (<adj><pprs>) are outside the UniMorph
verb-slot set scored here and dropped.

Usage: lt-expand apertium-nob.nob.dix | python3 scripts/nob/apertium_to_tsv.py
"""
import sys


def bundle(tags):
    t = set(tags)
    if "vblex" not in t:
        return None
    if "pasv" in t:
        return "V;IND;PASS" if "pres" in t else None
    if t == {"vblex", "pres"}:
        return "V;IND;PRS"
    if t == {"vblex", "pret"}:
        return "V;IND;PST"
    if t == {"vblex", "pp"}:
        return "V.PTCP;PST"
    if t == {"vblex", "imp"}:
        return "V;IMP"
    return None


def main():
    rows = set()
    for line in sys.stdin:
        line = line.rstrip("\n")
        if "<vblex>" not in line or ":" not in line:
            continue
        surface, lexical = line.split(":", 1)
        if lexical[:2] in (">:", "<:"):
            lexical = lexical[2:]
        if "+" in lexical:  # multi-part composites
            continue
        lemma, _, rest = lexical.partition("<")
        tags = [t for t in ("<" + rest).replace(">", " ").split("<") if t.strip()]
        b = bundle([t.strip() for t in tags])
        if not b or not lemma or not surface:
            continue
        rows.add((lemma, surface, b))
    out = sys.stdout
    for lemma, form, feat in sorted(rows):
        out.write(f"{lemma}\t{form}\t{feat}\n")


if __name__ == "__main__":
    main()
