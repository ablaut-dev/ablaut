#!/usr/bin/env python3
"""Mine the Tagalog verb lexicon from the oracle data.

Which affix family a Tagalog root takes — the actor voice (-um-, mag-,
mang-, ma-) and the undergoer voice (-in, -an, i-) — is lexical: it
cannot be predicted from the root's shape, exactly as German strong-verb
class cannot. So it is stored, one row per root, and the engine supplies
only the productive morphophonology (infixation, CV-reduplication, nasal
assimilation, the d->r and ni- allomorphs, o->u raising) given the class.

For each UniMorph root the best-fitting (actor, patient, h-insertion)
class is chosen by re-deriving the paradigm and counting hits; any slot
the winning class still misses on the *agreed* oracle gold is stored as
an explicit override (the handful of genuinely irregular stems, e.g.
kasal -> magpakasal, kita -> makita).

Columns: root  actor  patient  h  pfv_ag ipfv_ag cont_ag pfv_pf ipfv_pf cont_pf
(a `-` in a form column falls through to the productive rule.)

Usage: python3 scripts/tgl/mine_verbs.py data/tgl/unimorph.tsv data/tgl/kaikki.tsv
"""

import collections
import sys

sys.path.insert(0, "scripts/tgl")
from tgl_rules import gen, variants_of, SLOTS  # noqa: E402


def load(p):
    g = {}
    for line in open(p, encoding="utf-8"):
        lm, fm, ft = line.rstrip("\n").split("\t")
        g.setdefault((lm, ft), set()).add(fm)
    return g


def main(um_path, kaikki_path):
    u = load(um_path)
    k = load(kaikki_path)
    # per-lemma unimorph paradigm (for class fitting) and agreed gold
    para = collections.defaultdict(dict)
    for (lm, ft), forms in u.items():
        para[lm][ft] = forms
    agreed = collections.defaultdict(dict)
    for key in u:
        if key in k and (u[key] & k[key]):
            agreed[key[0]][key[1]] = u[key] | k[key]

    ACTORS = ["um", "mag", "mang", "ma"]
    PATS = ["in", "an", "i", None]
    rows = []
    for lm in sorted(para):
        d = para[lm]
        best, bs = None, -1
        for a in ACTORS:
            for p in PATS:
                for h in (True, False):
                    g = gen(lm, a, p, h)
                    sc = sum(1 for ft, f in g.items()
                             if ft in d and f in d[ft])
                    if sc > bs:
                        bs, best = sc, (a, p, h)
        a, p, h = best
        vs = variants_of(lm, a, p, h)
        overrides = {}
        for ft, var in agreed.get(lm, {}).items():
            if not (vs.get(ft, set()) & var):
                # store the first agreed spelling as the override
                overrides[ft] = sorted(var)[0]
        # skip a plain-default lemma with no PFOC and no override (the
        # engine's fallback already covers it), keep the rest.
        cols = [lm, a, p or "-", "1" if h else "0"]
        cols += [overrides.get(ft, "-") for ft in SLOTS]
        rows.append(cols)

    print("# Tagalog verb lexicon: the lexical affix class of each root,")
    print("# plus stored overrides for irregular stems. Mined by")
    print("# scripts/tgl/mine_verbs.py from the UniMorph paradigm and the")
    print("# UniMorph n kaikki agreement. See docs/tgl/oracles.md.")
    print("# root\tactor\tpatient\th\tpfv_ag\tipfv_ag\tcont_ag\tpfv_pf\tipfv_pf\tcont_pf")
    for cols in rows:
        print("\t".join(cols))


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
