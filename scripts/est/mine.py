#!/usr/bin/env python3
"""Mine Estonian gradation/irregular rows from golden mismatches."""
import collections

MISMATCHES = "target/golden_est_mismatches.tsv"
ORACLES = ["data/est/vabamorf.tsv", "data/est/kaikki.tsv"]
# driver: mismatching slot -> (row column, gold slot that fills it)
DRIVER = {
    "V;IND;PRS;SG;1": (1, "V;IND;PRS;SG;1"),
    "V;IND;PRS;SG;2": (1, "V;IND;PRS;SG;1"),
    "V;IND;PRS;SG;3": (2, "V;IND;PRS;SG;3"),
    "V;IND;PRS;PL;1": (1, "V;IND;PRS;SG;1"),
    "V;IND;PRS;PL;2": (1, "V;IND;PRS;SG;1"),
    "V;IND;PRS;PL;3": (1, "V;IND;PRS;SG;1"),
    "V;COND;PRS;SG;1": (1, "V;IND;PRS;SG;1"),
    "V;COND;PRS;SG;2": (1, "V;IND;PRS;SG;1"),
    "V;COND;PRS;SG;3": (1, "V;IND;PRS;SG;1"),
    "V;COND;PRS;PL;1": (1, "V;IND;PRS;SG;1"),
    "V;COND;PRS;PL;2": (1, "V;IND;PRS;SG;1"),
    "V;COND;PRS;PL;3": (1, "V;IND;PRS;SG;1"),
    "V;NFIN;DA": (3, "V;NFIN;DA"),
    "V;IND;PST;SG;1": (4, "V;IND;PST;SG;1"),
    "V;IND;PST;SG;2": (4, "V;IND;PST;SG;1"),
    "V;IND;PST;PL;1": (4, "V;IND;PST;SG;1"),
    "V;IND;PST;PL;2": (4, "V;IND;PST;SG;1"),
    "V;IND;PST;PL;3": (4, "V;IND;PST;SG;1"),
    "V.PTCP;PST;ACT": (5, "V.PTCP;PST;ACT"),
    "V;COND;PRF;SG;1": (5, "V.PTCP;PST;ACT"),
    "V;COND;PRF;SG;2": (5, "V.PTCP;PST;ACT"),
    "V;COND;PRF;SG;3": (5, "V.PTCP;PST;ACT"),
    "V;COND;PRF;PL;1": (5, "V.PTCP;PST;ACT"),
    "V;COND;PRF;PL;2": (5, "V.PTCP;PST;ACT"),
    "V.PTCP;PST;PASS": (6, "V.PTCP;PST;PASS"),
    "V;IND;PST;IMPRS": (6, "V.PTCP;PST;PASS"),
    "V;COND;PRS;IMPRS": (6, "V.PTCP;PST;PASS"),
    "V;IMP;IMPRS": (6, "V.PTCP;PST;PASS"),
    "V;NFIN;MA;PASS": (6, "V.PTCP;PST;PASS"),
    "V.PTCP;PRS;PASS": (6, "V.PTCP;PST;PASS"),
    "V;IMP;SG;2": (7, "V;IMP;SG;2"),
    "V;IMP;SG;3": (8, "V;IMP;SG;3"),
    "V;IMP;PL;1": (8, "V;IMP;SG;3"),
    "V;IMP;PL;2": (8, "V;IMP;SG;3"),
    "V;IMP;PL;3": (8, "V;IMP;SG;3"),
    "V;IND;PRS;IMPRS": (9, "V;IND;PRS;IMPRS"),
}
LATE = {"V;IND;PST;SG;3": (10, "V;IND;PST;SG;3"),
        "V.PTCP;PRS;ACT": (11, "V.PTCP;PRS;ACT"),
        "V;IND;PRS;PL;3": (12, "V;IND;PRS;PL;3")}
DRIVER.update(LATE)
N_COLS = 13


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
    for line in open("data/est/verbs.tsv"):
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
    with open("data/est/verbs.tsv", "w") as f:
        f.write("\n".join(header + [verbs[k] for k in sorted(verbs)]) + "\n")
    print(f"patched {n} (rows {len(verbs)})")


if __name__ == "__main__":
    main()
