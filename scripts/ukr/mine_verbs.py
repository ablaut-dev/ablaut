#!/usr/bin/env python3
"""Mine the Ukrainian explicit-form lexicon (data/ukr/verbs.tsv) from the
agreement of the two oracles (kaikki.org ∩ VESUM), restricted to the
lemmas the rule engine gets wrong (scripts/ukr/irregulars.txt, captured by
scripts/ukr/capture_irregulars.sh).

Each stored row carries the six non-past forms, the three imperative
forms and the four l-past forms; a slot the oracles agree on is stored as
their shared spelling, a slot only one oracle covers as that oracle's, and
a slot neither covers (or where they disagree, which the harness excludes
from gold anyway) is left as "-" to fall through to the rule engine.

Run: python3 scripts/ukr/mine_verbs.py
"""
import collections

COLS = [
    ("V;IND;PRS;SG;1", 1), ("V;IND;PRS;SG;2", 2), ("V;IND;PRS;SG;3", 3),
    ("V;IND;PRS;PL;1", 4), ("V;IND;PRS;PL;2", 5), ("V;IND;PRS;PL;3", 6),
    ("V;IMP;SG;2", 7), ("V;IMP;PL;1", 8), ("V;IMP;PL;2", 9),
    ("V;IND;PST;MASC;SG", 10), ("V;IND;PST;FEM;SG", 11),
    ("V;IND;PST;NEUT;SG", 12), ("V;IND;PST;PL", 13),
]


def load(path):
    g = collections.defaultdict(lambda: collections.defaultdict(set))
    for line in open(path, encoding="utf-8"):
        lemma, form, feat = line.rstrip("\n").split("\t")
        g[lemma][feat].add(form)
    return g


kaikki = load("data/ukr/kaikki.tsv")
vesum = load("data/ukr/vesum.tsv")


def pick(lemma, feat):
    """One canonical form for a slot: a spelling both oracles share wins
    (so the stored form is agreed and therefore scored); else whichever
    oracle covers it, VESUM first."""
    k = kaikki.get(lemma, {}).get(feat, set())
    v = vesum.get(lemma, {}).get(feat, set())
    both = k & v
    if both:
        return sorted(both)[0]
    if v:
        return sorted(v)[0]
    if k:
        return sorted(k)[0]
    return "-"


err = sorted({l.strip() for l in open("scripts/ukr/irregulars.txt") if l.strip()
              and not l.startswith("#")})

rows = []
for lemma in err:
    if lemma not in kaikki and lemma not in vesum:
        continue
    cells = [lemma] + ["-"] * 13
    for feat, col in COLS:
        cells[col] = pick(lemma, feat)
    rows.append("\t".join(cells))

hdr = ("# Ukrainian irregular verbs — mined from the kaikki ∩ VESUM oracle\n"
       "# agreement (scripts/ukr/mine_verbs.py). Rows the rule engine misses.\n"
       "# Regenerate: sh scripts/ukr/capture_irregulars.sh && "
       "python3 scripts/ukr/mine_verbs.py\n"
       "# lemma\tpres1sg\tpres2sg\tpres3sg\tpres1pl\tpres2pl\tpres3pl"
       "\timp2sg\timp1pl\timp2pl\tpastM\tpastF\tpastN\tpastPl\n")
open("data/ukr/verbs.tsv", "w", encoding="utf-8").write(hdr + "\n".join(rows) + "\n")
print(f"erroring lemmas: {len(err)}, stored rows: {len(rows)}")
