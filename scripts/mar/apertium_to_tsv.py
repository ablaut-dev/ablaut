#!/usr/bin/env python3
"""Convert `lt-expand` output of apertium-mar into the common
golden-harness TSV: `lemma <TAB> form <TAB> features`.

apertium-mar is the primary oracle: a hand-built lttoolbox morphological
dictionary with no Wiktionary lineage. `lt-expand` walks it into
`surface:lemma<tags>` pairs (see scripts/mar/fetch_apertium.sh); this
maps its finite tag set onto the UniMorph-style bundle the engine and the
kaikki adapter share, so the harness can score and intersect the two.

Tag mapping (apertium → shared bundle), person p1/p2/p3, gender m/f/nt,
number sg/pl:
  inf                         → V;NFIN                    (करणे)
  trans + perf                → V;CVB;PFV                 completive converb (करून)
  pros + mfn + sp             → V;PROSP                   prospective (करणार)
  sup                         → V;PURP                    purposive (करायला)
  pres + impf + p/g/n         → V;IND;PRS;HAB;p;g;n       present habitual (करतो)
  perf + p/g/n                → V;IND;PST;PFV;p;g;n       perfective / simple past (केला)
  subj + g/n                  → V;SBJV;g;n                subjunctive, person-invariant (करावा)
  fut + p/n                   → V;IND;FUT;p;n             future, no gender (करेन)
  imp + p/n                   → V;IMP;p;n                 imperative (कर)

The emphatic clitics (+च/+ही), the case-marked gerunds and the perfect/
present participles (केलेला / करणारा) apertium also expands are outside
the bounded cell set and dropped. A handful of apertium data bugs are
dropped too (see BAD_LEMMAS / drop()).

Usage: lt-expand apertium-mar.mar.dix | python3 scripts/mar/apertium_to_tsv.py
"""
import sys

G = {"m": "MASC", "f": "FEM", "nt": "NEUT", "mf": "MF", "mfn": "MFN"}
N = {"sg": "SG", "pl": "PL"}
P = {"p1": "1", "p2": "2", "p3": "3"}

# apertium-mar dictionary bugs: lemmas whose citation is internally
# inconsistent with the forms the entry lists, so no engine can match them.
#   सांगसांगणे  — the stem is doubled in the lemma key (the forms are
#                 सांगितल-, i.e. the real verb सांगणे)
#   लिहीणे      — spelled with दीर्घ ई but the forms use ह्रस्व इ (लिहि-)
#   रहाणे/पहाणे/वहाणे — cited with the हा stem but the forms use the
#                 metathesised राह/पाह/वाह (the standard lemmas are
#                 राहणे/पाहणे/वाहणे)
#   णे          — a bare, stemless garbage entry
BAD_LEMMAS = {"सांगसांगणे", "लिहीणे", "रहाणे", "पहाणे", "वहाणे", "णे"}
LIGHT_VERBS = ("करणे", "होणे", "देणे", "घेणे")


def bundle(tags):
    t = set(tags)
    if "vblex" not in t:
        return None
    if t == {"vblex", "inf"}:
        return "V;NFIN"
    if "trans" in t and "perf" in t and "past" not in t and not (t & set(P)):
        return "V;CVB;PFV"
    if "pros" in t and "mfn" in t and "sp" in t and not (t & set(P)):
        return "V;PROSP"
    if t == {"vblex", "sup"}:
        return "V;PURP"
    p = next((P[x] for x in tags if x in P), None)
    g = next((G[x] for x in tags if x in G), None)
    n = next((N[x] for x in tags if x in N), None)
    if "imp" in t and p and n:
        return f"V;IMP;{p};{n}"
    if "fut" in t and p and n:
        return f"V;IND;FUT;{p};{n}"
    if "subj" in t and g and n:  # subjunctive does not distinguish person
        return f"V;SBJV;{g};{n}"
    if "pres" in t and "impf" in t and p and g and n:
        return f"V;IND;PRS;HAB;{p};{g};{n}"
    if "perf" in t and "past" not in t and "pprs" not in t and p and g and n:
        return f"V;IND;PST;PFV;{p};{g};{n}"
    return None


def drop(lemma, b):
    if lemma in BAD_LEMMAS:
        return True
    # no-space light-verb compound glued into one token (प्राप्तहोणे for
    # प्राप्त होणे): the last verb is not separable, so it would be
    # conjugated as if simplex. Space-separated compounds are kept.
    if " " not in lemma and lemma not in LIGHT_VERBS and lemma.endswith(LIGHT_VERBS):
        return True
    # apertium spells the होणे subjunctive with the non-standard हो-stem
    # (होवा); standard Marathi is the व्हा-stem (व्हावा), on which the
    # engine and kaikki agree. Drop apertium's होणे-family subjunctive.
    if b.startswith("V;SBJV") and lemma.split()[-1] == "होणे":
        return True
    return False


def main():
    rows = set()
    for line in sys.stdin:
        line = line.rstrip("\n")
        if "<vblex>" not in line or ":" not in line:
            continue
        surface, lexical = line.split(":", 1)
        # lt-expand marks direction-restricted entries `surface:>:lemma…`
        # (RL-only) or `surface:<:lemma…` (LR-only); drop the marker.
        if lexical[:2] in (">:", "<:"):
            lexical = lexical[2:]
        # drop the emphatic-clitic and other multi-part composites
        if "+" in lexical:
            continue
        lemma, _, rest = lexical.partition("<")
        tags = [t.strip() for t in ("<" + rest).replace(">", " ").split("<") if t.strip()]
        b = bundle(tags)
        if not b or not lemma or not surface:
            continue
        if drop(lemma, b):
            continue
        rows.add((lemma, surface, b))
    out = sys.stdout
    for lemma, form, feat in sorted(rows):
        out.write(f"{lemma}\t{form}\t{feat}\n")


if __name__ == "__main__":
    main()
