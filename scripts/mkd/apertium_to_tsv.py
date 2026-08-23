#!/usr/bin/env python3
"""Convert `lt-expand` output of apertium-mkd into the common golden TSV.

apertium-mkd is the independent oracle: a hand-built lttoolbox dictionary
with no Wiktionary lineage. `lt-expand` walks it into `surface:lemma<tags>`
pairs; this maps its <vblex> tag set onto the UniMorph-style bundle the
engine and the UniMorph adapter share. Macedonian has no infinitive — the
lemma key is the 3sg present.

Tag mapping (apertium <vblex> → shared bundle):
  pres p{1,2,3} {sg,pl}        → V;PRS;{p};{SG,PL}
  pii  p{1,2,3} {sg,pl}        → V;PROG;PST;{p};{SG,PL}      (imperfect)
  aor  p{1,2,3} {sg,pl}        → V;PST;{p};{SG,PL}           (aorist)
  lp pii {m,f,nt,mfn+pl}       → V.PTCP;PROG;PST;{MASC,FEM,NEUT}/PL
  lp aor {m,f,nt,mfn+pl}       → V.PTCP;PST;{MASC,FEM,NEUT}/PL
  imp {sg,pl}                  → V;IMP;2;{SG,PL}
  pp m sg ind                  → V.PTCP;PST;PASS   (bare citation form)
  pprs adv                     → V.CVB             (adverbial participle)
The article-inflected participle forms (def/prx/dst and fem/neut/pl of pp)
are noun-like inflection and dropped. tv/iv (transitivity) is ignored.

Usage: lt-expand apertium-mkd.mkd.dix | python3 scripts/mkd/apertium_to_tsv.py
"""
import sys

P = {"p1": "1", "p2": "2", "p3": "3"}
LP = {"m": "MASC;SG", "f": "FEM;SG", "nt": "NEUT;SG", "mfn": "PL"}


def bundle(tags):
    t = set(tags)
    if "vblex" not in t:
        return None
    p = next((P[x] for x in tags if x in P), None)
    n = "SG" if "sg" in t else ("PL" if "pl" in t else None)
    if "lp" in t:
        g = next((LP[x] for x in tags if x in LP), None)
        if not g:
            return None
        base = "V.PTCP;PROG;PST" if "pii" in t else ("V.PTCP;PST" if "aor" in t else None)
        return f"{base};{g}" if base else None
    if "pp" in t:
        return "V.PTCP;PST;PASS" if (t >= {"m", "sg", "ind"}) else None
    if "pprs" in t and "adv" in t:
        return "V.CVB"
    if "imp" in t and n:
        return f"V;IMP;2;{n}"
    if "pres" in t and p and n:
        return f"V;PRS;{p};{n}"
    if "pii" in t and p and n:
        return f"V;PROG;PST;{p};{n}"
    if "aor" in t and p and n:
        return f"V;PST;{p};{n}"
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
        if "+" in lexical or not surface:
            continue
        lemma, _, rest = lexical.partition("<")
        tags = [x for x in ("<" + rest).replace(">", " ").split("<") if x.strip()]
        b = bundle([x.strip() for x in tags])
        if b and lemma:
            rows.add((lemma, surface, b))
    out = sys.stdout
    for lemma, form, feat in sorted(rows):
        out.write(f"{lemma}\t{form}\t{feat}\n")


if __name__ == "__main__":
    main()
