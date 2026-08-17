#!/usr/bin/env python3
"""Mine Irish class/stem rows from golden mismatches."""
import collections

MISMATCHES = "target/golden_gle_mismatches.tsv"
ORACLES = ["data/gle/bunamo.tsv", "data/gle/kaikki.tsv"]
DRIVER = {}
for p in ("BASE", "1SG", "2SG", "1PL", "2PL", "3PL"):
    DRIVER[f"V;PRS;{p}"] = (1, "V;PRS;BASE")
for p in ("BASE", "1SG", "2SG", "1PL", "2PL", "3PL", "AUTO"):
    DRIVER[f"V;PST;{p}"] = (2, "V;PST;BASE")
    DRIVER[f"V;FUT;{p}"] = (3, "V;FUT;BASE")
    DRIVER[f"V;COND;{p}"] = (3, "V;FUT;BASE")
    DRIVER[f"V;PSTHAB;{p}"] = (1, "V;PRS;BASE")
    DRIVER[f"V;SBJV;{p}"] = (1, "V;PRS;BASE")
    DRIVER[f"V;IMP;{p}"] = (1, "V;PRS;BASE")
DRIVER["V;VN"] = (4, "V;VN")
DRIVER["V.PTCP"] = (5, "V.PTCP")
DRIVER["V;IMP;2SG"] = (6, "V;IMP;2SG")
DRIVER["V;IMP;2PL"] = (7, "V;IMP;2PL")
for f in ("V;PRS;AUTO", "V;PSTHAB;AUTO", "V;PSTHAB;2SG", "V;IMP;AUTO",
          "V;SBJV;AUTO"):
    DRIVER[f] = (8, "V;PRS;AUTO")
N_COLS = 9


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
    for line in open("data/gle/verbs.tsv"):
        line = line.rstrip("\n")
        if line.startswith("#"):
            header.append(line)
        elif line:
            verbs[line.split("\t")[0]] = line
    n = 0
    for line in open(MISMATCHES):
        f = line.rstrip("\n").split("\t")
        if len(f) < 2 or f[1] not in DRIVER:
            continue
        lemma, feat = f[0], f[1]
        i, src = DRIVER[feat]
        g = gold.get(lemma, {}).get(src)
        if not g:
            continue
        row = verbs.get(lemma, lemma + "\t" + "\t".join(["-"] * (N_COLS - 1)))
        cols = row.split("\t")
        while len(cols) < N_COLS:
            cols.append("-")
        if cols[i] == "-":
            cols[i] = pick(*g)
            verbs[lemma] = "\t".join(cols)
            n += 1
    with open("data/gle/verbs.tsv", "w") as f:
        f.write("\n".join(header + [verbs[k] for k in sorted(verbs)]) + "\n")
    print(f"patched {n} (rows {len(verbs)})")


if __name__ == "__main__":
    main()
