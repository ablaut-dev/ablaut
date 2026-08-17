#!/usr/bin/env python3
"""Mine English irregular/doubling rows from golden mismatches."""
import collections

MISMATCHES = "target/golden_eng_mismatches.tsv"
ORACLES = ["data/eng/agid.tsv", "data/eng/kaikki.tsv"]
COL = {"V;PST": 1, "V.PTCP;PST": 2, "V.PTCP;PRS": 3, "V;PRS;3;SG": 4}


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
    for line in open("data/eng/verbs.tsv"):
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
        row = verbs.get(lemma, lemma + "\t-\t-\t-\t-")
        cols = row.split("\t")
        i = COL[feat]
        if cols[i] == "-":
            cols[i] = pick(*g)
            verbs[lemma] = "\t".join(cols)
            n += 1
    with open("data/eng/verbs.tsv", "w") as f:
        f.write("\n".join(header + [verbs[k] for k in sorted(verbs)]) + "\n")
    print(f"patched {n} (rows {len(verbs)})")


if __name__ == "__main__":
    main()
