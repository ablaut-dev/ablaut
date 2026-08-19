#!/usr/bin/env python3
"""Mine the Dutch irregular-verb lexicon (data/nld/verbs.tsv) from the
two-oracle agreement, restricted to the lemmas the rule engine gets
wrong (scripts/nld/irregulars.txt, captured against an empty lexicon).

For each such lemma the agreed paradigm is copied into the columns the
Dutch engine reads: the stem (= present 1sg), the six present forms,
the number-marked past, the two participles and the imperative. Columns
the rule engine would derive correctly are still stored — harmless, and
it keeps every mined lemma exact. Run after a golden_nld pass; a stable
irregulars.txt keeps the mine reproducible.
"""
import collections


def load(path):
    g = collections.defaultdict(lambda: collections.defaultdict(set))
    for line in open(path, encoding="utf-8"):
        lemma, form, feat = line.rstrip("\n").split("\t")
        g[lemma][feat].add(form)
    return g


ka = load("data/nld/kaikki.tsv")
ap = load("data/nld/apertium.tsv")


def agreed(lemma, feat):
    """The canonical agreed form for a slot: an oracle-shared form wins,
    else kaikki (the more reliable oracle), else Apertium."""
    k = ka.get(lemma, {}).get(feat, set())
    a = ap.get(lemma, {}).get(feat, set())
    both = k & a
    if both:
        return sorted(both)[0]
    if k:
        return sorted(k)[0]
    if a:
        return sorted(a)[0]
    return None


def cell(lemma, feat):
    return agreed(lemma, feat) or "-"


irregulars = sorted(
    {l.strip() for l in open("scripts/nld/irregulars.txt") if l.strip() and not l.startswith("#")}
)

PRES = [
    "V;IND;PRS;SG;1",
    "V;IND;PRS;SG;2",
    "V;IND;PRS;SG;3",
    "V;IND;PRS;PL;1",
    "V;IND;PRS;PL;2",
    "V;IND;PRS;PL;3",
]

rows = []
for lemma in irregulars:
    if lemma not in ka and lemma not in ap:
        continue
    stem = agreed(lemma, "V;IND;PRS;SG;1")
    pres_forms = [agreed(lemma, f) for f in PRES]
    pres = ",".join(pres_forms) if all(pres_forms) else "-"
    row = [
        lemma,
        stem or "-",
        pres,
        cell(lemma, "V;IND;PST;SG"),
        cell(lemma, "V;IND;PST;PL"),
        cell(lemma, "V.PTCP;PST"),
        cell(lemma, "V;IMP"),
        cell(lemma, "V.PTCP;PRS"),
    ]
    rows.append("\t".join(row))

# Core auxiliaries and modals Apertium keeps outside vblex are supplied
# by hand from kaikki (scripts/nld/manual.tsv), appended verbatim.
mined_lemmas = {r.split("\t", 1)[0] for r in rows}
manual = []
for line in open("scripts/nld/manual.tsv", encoding="utf-8"):
    if line.startswith("#") or not line.strip():
        continue
    if line.split("\t", 1)[0] not in mined_lemmas:
        manual.append(line.rstrip("\n"))
rows = sorted(rows + manual)

hdr = (
    "# Dutch irregular verbs — mined from the kaikki ∩ Apertium agreement,\n"
    "# restricted to the lemmas the rule engine misses (empty-lexicon capture,\n"
    "# scripts/nld/capture_irregulars.sh). Exact match.\n"
    "# Regenerate: python3 scripts/nld/mine_verbs.py\n"
    "# lemma\tstem\tpres\tpast_sg\tpast_pl\tpp\timp\tprespart\n"
)
open("data/nld/verbs.tsv", "w", encoding="utf-8").write(hdr + "\n".join(rows) + "\n")
print(f"irregular lemmas: {len(irregulars)}, stored: {len(rows)}")
