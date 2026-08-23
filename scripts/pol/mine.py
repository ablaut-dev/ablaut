#!/usr/bin/env python3
"""Mine the Polish exception table from the two oracle TSVs.

Polish is cited by its infinitive. Two things are productive and generated
by rule (see productive(), mirrored in src/pol.rs): the infinitive itself,
and the gendered past l-form, which for the regular -ać/-ić/-yć/-uć verbs
is the infinitive minus -ć plus -ł/-ła/-ło (sg m/f/n), -li (virile pl) and
-ły (non-virile pl): robić → robił/robiła/robiło/robili/robiły. Everything
class-dependent — the present, the synthetic (perfective) future, the
imperative — and the irregular pasts (-eć→-ał, -ść, -c, suppletives) are
mined. Where the two oracles attest the same form we store that; otherwise
the UniMorph reading.

Usage: mine.py data/pol/unimorph.tsv data/pol/sgjp.tsv > data/pol/parts.tsv
"""
import sys
from collections import defaultdict

CELLS = [
    "V;PRS;1;SG", "V;PRS;2;SG", "V;PRS;3;SG", "V;PRS;1;PL", "V;PRS;2;PL", "V;PRS;3;PL",
    "V;FUT;1;SG", "V;FUT;2;SG", "V;FUT;3;SG", "V;FUT;1;PL", "V;FUT;2;PL", "V;FUT;3;PL",
    "V;PST;3;SG;MASC", "V;PST;3;SG;FEM", "V;PST;3;SG;NEUT",
    "V;PST;3;PL;MASC;HUM", "V;PST;3;PL",
    "V;IMP;2;SG", "V;IMP;1;PL", "V;IMP;2;PL", "V;NFIN",
]

PAST = {
    "V;PST;3;SG;MASC": "ł", "V;PST;3;SG;FEM": "ła", "V;PST;3;SG;NEUT": "ło",
    "V;PST;3;PL;MASC;HUM": "li", "V;PST;3;PL": "ły",
}


def productive(cit, feat):
    if feat == "V;NFIN":
        return cit
    if feat in PAST and cit.endswith("ć"):
        return cit[:-1] + PAST[feat]
    return None


def main(uni_path, sgjp_path):
    def read(path):
        d = defaultdict(set)
        for l in open(path):
            a = l.rstrip("\n").split("\t")
            if len(a) >= 3 and a[2].startswith("V"):
                d[(a[0], a[2])].add(a[1])
        return d
    uni, sgjp = read(uni_path), read(sgjp_path)

    def chosen(lemma, feat):
        a, b = uni.get((lemma, feat), set()), sgjp.get((lemma, feat), set())
        pool = (a & b) or a or b
        return sorted(pool)[0] if pool else None

    lemmas = sorted({l for (l, f) in list(uni) + list(sgjp) if f == "V;NFIN"})
    print("# lemma(infinitive)\t" + "\t".join(CELLS) + '  ("-" = productive default)')
    for lemma in lemmas:
        if not lemma or " " in lemma:
            continue
        out = []
        for c in CELLS:
            f = chosen(lemma, c)
            out.append(f if f and f != productive(lemma, c) else "-")
        if any(x != "-" for x in out):
            print(lemma + "\t" + "\t".join(out))


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
