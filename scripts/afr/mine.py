#!/usr/bin/env python3
"""Mine the Afrikaans exception table from the two oracle TSVs.

The engine's productive rules are: present = infinitive; past participle
= ge- + stem (with geë- before a stem-initial e); present participle =
stem + -ende; no synthetic past. Everything that deviates — inseparable
prefix verbs that take no ge- (verstaan -> verstaan), separable-prefix
verbs whose ge- goes inside (aankom -> aangekom), spelling alternations
in the -ende participle (loop -> lopende), and the closed set of
preterites (wees -> was) — is stored here, matched exactly. Where the
two oracles attest the same form we store that; otherwise we store the
UniMorph reading. Regular verbs need no row.

Usage: mine.py data/afr/unimorph.tsv data/afr/kaikki.tsv > data/afr/parts.tsv
"""
import sys
from collections import defaultdict


def read(path):
    d = defaultdict(set)
    for line in open(path):
        parts = line.rstrip("\n").split("\t")
        if len(parts) < 3:
            continue
        lemma, form, feat = parts[0], parts[1], parts[2]
        if not feat.startswith("V"):
            continue
        d[(lemma, feat)].add(form)
    return d


def ge_rule(lemma):
    if lemma[:1] == "e":
        return "geë" + lemma[1:]
    return "ge" + lemma


def ende_rule(lemma):
    return lemma + "ende"


def main(uni_path, kai_path):
    uni = read(uni_path)
    kai = read(kai_path)
    lemmas = sorted({l for (l, _f) in uni} | {l for (l, _f) in kai})

    def chosen(lemma, feat):
        a, b = uni.get((lemma, feat), set()), kai.get((lemma, feat), set())
        agreed = a & b
        pool = agreed or a or b
        return sorted(pool)[0] if pool else None

    print("# lemma\tpresent\tpast\tpast-participle\tpresent-participle\timperative"
          "  (\"-\" = productive default)")
    for lemma in lemmas:
        if not lemma or not lemma[0].isalpha():
            continue
        present = chosen(lemma, "V;PRS")
        ptcp_pst = chosen(lemma, "V.PTCP;PST")
        ptcp_prs = chosen(lemma, "V.PTCP;PRS")
        past = chosen(lemma, "V;PST")
        imper = chosen(lemma, "V;IMP")

        c_present = present if present and present != lemma else "-"
        c_past = past if past else "-"
        c_ptcp = ptcp_pst if ptcp_pst and ptcp_pst != ge_rule(lemma) else "-"
        c_prs = ptcp_prs if ptcp_prs and ptcp_prs != ende_rule(lemma) else "-"
        c_imp = imper if imper and imper != lemma else "-"

        if any(c != "-" for c in (c_present, c_past, c_ptcp, c_prs, c_imp)):
            print(f"{lemma}\t{c_present}\t{c_past}\t{c_ptcp}\t{c_prs}\t{c_imp}")


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
