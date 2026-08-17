#!/usr/bin/env python3
"""Mine Slovenian class assignments and explicit rows from golden
mismatches. Classes cover the regular presents; explicit rows carry
stem alternations. Iterate with golden_ces until dry."""
import collections

MISMATCHES = "target/golden_slv_mismatches.tsv"
ORACLES = ["data/slv/sloleks.tsv", "data/slv/kaikki.tsv"]

PRS = [f"V;IND;PRS;{n};{p}" for n in ("SG", "DU", "PL")
       for p in (1, 2, 3)]
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


E9 = {"a": ["am", "aš", "a", "ava", "ata", "ata", "amo", "ate", "ajo"],
      "i": ["im", "iš", "i", "iva", "ita", "ita", "imo", "ite", "ijo"],
      "e": ["em", "eš", "e", "eva", "eta", "eta", "emo", "ete", "ejo"]}


def class_present(inf, cls):
    if cls == "uje" and (inf.endswith("ovati") or inf.endswith("evati")):
        return [inf[:-5] + "uj" + e for e in E9["e"]]
    if cls == "a" and inf.endswith("ti"):
        return [inf[:-2] + e for e in E9["a"]]
    if cls == "i" and (inf.endswith("iti") or inf.endswith("eti")):
        return [inf[:-3] + e for e in E9["i"]]
    if cls == "ne" and inf.endswith("iti"):
        return [inf[:-3] + e for e in E9["e"]]
    return None


def pick(variants, shared):
    return sorted(shared or variants)[0]


def main():
    gold = load_gold()
    lemmas = sorted({l.split("\t")[0] for l in open(MISMATCHES) if l.strip()})
    ch, classes = read("data/slv/classes.tsv")
    vh, verbs = read("data/slv/verbs.tsv")
    n_class = n_patch = 0
    for lemma in lemmas:
        feats = gold.get(lemma)
        if not feats or lemma in verbs or lemma in classes:
            continue
        prs = [feats.get(k) for k in PRS]
        chosen = None
        for cls in ("uje", "a", "i", "ne"):
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
        elif feat.startswith("V;IMP"):
            i, src = 10, "V;IMP;SG;2"
        elif feat.startswith("V.PTCP;PST"):
            row0 = verbs.get(lemma, "")
            cols0 = row0.split("\t") if row0 else []
            if len(cols0) > 11 and cols0[11] != "-" and feat != "V.PTCP;PST;M;SG":
                i, src = 13, "V.PTCP;PST;F;SG"
            else:
                i, src = 11, "V.PTCP;PST;M;SG"
        elif feat == "V;SUP":
            i, src = 12, "V;SUP"
        else:
            continue
        g = feats.get(src)
        if not g and src == "V.PTCP;PST;M;SG":
            # No masculine base in gold (reči) — carry the feminine.
            i, src = 13, "V.PTCP;PST;F;SG"
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
    write("data/slv/classes.tsv", ch, classes)
    write("data/slv/verbs.tsv", vh, verbs)
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
