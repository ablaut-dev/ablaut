#!/usr/bin/env python3
"""Mine Danish principal-parts rows from golden mismatches."""
import collections

MISMATCHES = "target/golden_dan_mismatches.tsv"
ORACLES = ["data/dan/cor-verbs.tsv", "data/dan/kaikki.tsv"]
COL = {"V;PRS;ACT": 1, "V;PST;ACT": 2, "V.PTCP;PST": 3, "V;IMP": 4, "V.PTCP;PRS": 5, "V;PST;PASS": 6}


def load_gold():
    golds = []
    for path in ORACLES:
        g = collections.defaultdict(dict)
        for line in open(path):
            f = line.rstrip("\n").split("\t")
            if len(f) >= 3:
                g[f[0]].setdefault(f[2], set()).add(f[1])
        golds.append(g)
    a, b = golds
    gold = collections.defaultdict(dict)
    for lemma, feats in a.items():
        for feat, va in feats.items():
            vb = b.get(lemma, {}).get(feat)
            if vb is None:
                gold[lemma][feat] = (va, va)
            elif va & vb:
                gold[lemma][feat] = (va | vb, va & vb)
    return gold


def pick(variants, shared):
    return sorted(shared or variants)[0]


def main():
    gold = load_gold()
    verbs = {}
    header = []
    for line in open("data/dan/parts.tsv"):
        line = line.rstrip("\n")
        if line.startswith("#"):
            header.append(line)
        elif line:
            verbs[line.split("\t")[0]] = line
    n = 0
    for line in open(MISMATCHES):
        f = line.rstrip("\n").split("\t")
        if len(f) < 2 or f[1] not in COL:
            continue
        lemma, feat = f[0], f[1]
        g = gold.get(lemma, {}).get(feat)
        if not g:
            continue
        row = verbs.get(lemma, lemma + "\t-\t-\t-\t-\t-\t-")
        cols = row.split("\t")
        while len(cols) < 7:
            cols.append("-")
        i = COL[feat]
        if cols[i] == "-":
            cols[i] = pick(*g)
            verbs[lemma] = "\t".join(cols)
            n += 1
    with open("data/dan/parts.tsv", "w") as f:
        f.write("\n".join(header + [verbs[k] for k in sorted(verbs)]) + "\n")
    print(f"patched {n} (rows {len(verbs)})")


if __name__ == "__main__":
    main()
