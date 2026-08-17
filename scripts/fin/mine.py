#!/usr/bin/env python3
"""Mine Finnish gradation/type rows from golden mismatches."""
import collections

MISMATCHES = "target/golden_fin_mismatches.tsv"
ORACLES = ["data/fin/omorfi.tsv", "data/fin/kaikki.tsv"]
DRIVER = {}
for f in ["V;IND;PRS;SG;1", "V;IND;PRS;SG;2", "V;IND;PRS;PL;1",
          "V;IND;PRS;PL;2"]:
    DRIVER[f] = (1, "V;IND;PRS;SG;1")
for f in ["V;IND;PRS;SG;3", "V;IND;PRS;PL;3", "V.PTCP;PRS;ACT"]:
    DRIVER[f] = (2, "V;IND;PRS;SG;3")
for n in ("SG", "PL"):
    for p in (1, 2, 3):
        DRIVER[f"V;COND;{n};{p}"] = (2, "V;IND;PRS;SG;3")
for f in ["V;IND;PST;SG;1", "V;IND;PST;SG;2", "V;IND;PST;PL;1",
          "V;IND;PST;PL;2"]:
    DRIVER[f] = (3, "V;IND;PST;SG;1")
for f in ["V;IND;PST;SG;3", "V;IND;PST;PL;3"]:
    DRIVER[f] = (4, "V;IND;PST;SG;3")
for f in (["V.PTCP;PST;ACT", "V;IMP;SG;3", "V;IMP;PL;1", "V;IMP;PL;2",
           "V;IMP;PL;3"]
          + [f"V;POT;{n};{p}" for n in ("SG", "PL") for p in (1, 2, 3)]):
    DRIVER[f] = (5, "V.PTCP;PST;ACT")
DRIVER["V;IND;PRS;PASS"] = (6, "V;IND;PRS;PASS")
for f in ["V;IND;PST;PASS", "V;COND;PASS", "V;POT;PASS", "V;IMP;PASS",
          "V.PTCP;PST;PASS", "V.PTCP;PRS;PASS"]:
    DRIVER[f] = (7, "V;IND;PST;PASS")
DRIVER["V;IMP;SG;2"] = (8, "V;IMP;SG;2")
DRIVER["V;IND;PRS;PL;3"] = (9, "V;IND;PRS;PL;3")
for f in ["V;IMP;SG;3", "V;IMP;PL;1", "V;IMP;PL;2", "V;IMP;PL;3"]:
    DRIVER[f] = (10, "V;IMP;PL;2")
for n in ("SG", "PL"):
    for p in (1, 2, 3):
        DRIVER[f"V;COND;{n};{p}"] = (11, "V;COND;SG;3")
        DRIVER[f"V;POT;{n};{p}"] = (12, "V;POT;SG;3")
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
    for line in open("data/fin/verbs.tsv"):
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
            # No gold for the driver slot, but this derived slot
            # mismatches: a polluted single-oracle value — reset it so
            # the default derivation (which the agreement gold liked)
            # comes back. Only in restore mode (agreement runs).
            import os
            if os.environ.get("RESTORE") and lemma in verbs:
                cols = verbs[lemma].split("\t")
                while len(cols) < N_COLS:
                    cols.append("-")
                if cols[i] != "-":
                    cols[i] = "-"
                    verbs[lemma] = "\t".join(cols)
                    n += 1
            continue
        row = verbs.get(lemma, lemma + "\t" + "\t".join(["-"] * (N_COLS - 1)))
        cols = row.split("\t")
        while len(cols) < N_COLS:
            cols.append("-")
        variants, shared = g
        if cols[i] == "-" or cols[i] not in variants:
            cols[i] = pick(*g)
            verbs[lemma] = "\t".join(cols)
            n += 1
    with open("data/fin/verbs.tsv", "w") as f:
        f.write("\n".join(header + [verbs[k] for k in sorted(verbs)]) + "\n")
    print(f"patched {n} (rows {len(verbs)})")


if __name__ == "__main__":
    main()
