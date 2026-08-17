#!/usr/bin/env python3
"""Mine Czech class assignments and explicit rows from golden
mismatches. Classes cover the regular presents; explicit rows carry
stem alternations. Iterate with golden_ces until dry."""
import collections

MISMATCHES = "target/golden_ces_mismatches.tsv"
ORACLES = ["data/ces/morfflex.tsv", "data/ces/kaikki.tsv"]

PRS = [f"V;IND;PRS;{n};{p}" for n, p in
       [("SG", 1), ("SG", 2), ("SG", 3), ("PL", 1), ("PL", 2), ("PL", 3)]]
# column per feature; participles and transgressives are driven by
# their base slot's gold form.
DRIVER = {"V;IMP;SG;2": (7, "V;IMP;SG;2"), "V;IMP;PL;1": (8, "V;IMP;PL;1"),
          "V;IMP;PL;2": (9, "V;IMP;PL;2")}
N_COLS = 15


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


def class_present(inf, cls):
    if cls == "uje" and inf.endswith("ovat"):
        s = inf[:-4]
        return [s + e for e in ("uji", "uješ", "uje", "ujeme", "ujete", "ují")]
    if cls == "a" and inf.endswith("at"):
        s = inf[:-2]
        return [s + e for e in ("ám", "áš", "á", "áme", "áte", "ají")]
    if cls == "i" and inf.endswith("it"):
        s = inf[:-2]
        return [s + e for e in ("ím", "íš", "í", "íme", "íte", "í")]
    if cls in ("et-i", "et-eji") and (inf.endswith("et") or inf.endswith("ět")):
        s = inf[:-2]
        last = "í" if cls == "et-i" else "ějí"
        return [s + e for e in ("ím", "íš", "í", "íme", "íte", last)]
    if cls == "ne" and inf.endswith("nout"):
        s = inf[:-4]
        return [s + e for e in ("nu", "neš", "ne", "neme", "nete", "nou")]
    return None


def pick(variants, shared):
    return sorted(shared or variants)[0]


def main():
    gold = load_gold()
    lemmas = sorted({l.split("\t")[0] for l in open(MISMATCHES) if l.strip()})
    ch, classes = read("data/ces/classes.tsv")
    vh, verbs = read("data/ces/verbs.tsv")
    n_class = n_patch = 0
    for lemma in lemmas:
        feats = gold.get(lemma)
        if not feats or lemma in verbs or lemma in classes:
            continue
        prs = [feats.get(k) for k in PRS]
        chosen = None
        for cls in ("uje", "a", "i", "et-i", "et-eji", "ne"):
            cf = class_present(lemma, cls)
            if cf and all(g is None or f in g[0] for f, g in zip(cf, prs)):
                chosen = cls
                break
        if chosen:
            classes[lemma] = f"{lemma}\t{chosen}"
            n_class += 1
    # Patch pass: fill row columns from the gold slots.
    for line in open(MISMATCHES):
        f = line.rstrip("\n").split("\t")
        if len(f) < 2:
            continue
        lemma, feat = f[0], f[1]
        feats = gold.get(lemma)
        if not feats:
            continue
        if feat in PRS:
            i, src = 1 + PRS.index(feat), feat
        elif feat in DRIVER:
            i, src = DRIVER[feat]
        elif feat.startswith("V.PTCP;PST"):
            row0 = verbs.get(lemma, "")
            cols0 = row0.split("\t") if row0 else []
            if len(cols0) > 10 and cols0[10] != "-" and not feat.endswith("MA;SG"):
                i, src = 14, "V.PTCP;PST;F;SG"
            else:
                i, src = 10, "V.PTCP;PST;MA;SG"
        elif feat.startswith("V.PTCP;PASS"):
            i, src = 11, "V.PTCP;PASS;MA;SG"
        elif feat == "V;CVB;PRS;M":
            i, src = 12, "V;CVB;PRS;M"
        elif feat.startswith("V;CVB;PRS"):
            i, src = 13, "V;CVB;PRS;FN"
        else:
            continue
        g = feats.get(src)
        if not g:
            continue
        row = verbs.get(lemma, lemma + "\t" + "\t".join(["-"] * (N_COLS - 1)))
        cols = row.split("\t")
        while len(cols) < N_COLS:
            cols.append("-")
        if cols[i] == "-":
            cols[i] = pick(*g)
            verbs[lemma] = "\t".join(cols)
            n_patch += 1
    write("data/ces/classes.tsv", ch, classes)
    write("data/ces/verbs.tsv", vh, verbs)
    print(f"classes +{n_class} (total {len(classes)}), "
          f"patched {n_patch} (rows {len(verbs)})")


def read(path):
    out, header = {}, []
    for line in open(path):
        line = line.rstrip("\n")
        if line.startswith("#"):
            header.append(line)
        elif line:
            out[line.split("\t")[0]] = line
    return header, out


def write(path, header, table):
    with open(path, "w") as f:
        f.write("\n".join(header + [table[k] for k in sorted(table)]) + "\n")


if __name__ == "__main__":
    main()
