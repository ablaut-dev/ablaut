#!/usr/bin/env python3
"""Mine Romanian class assignments and explicit rows from golden mismatches.

Reads target/golden_ron_mismatches.tsv plus the two oracle TSVs, and for
every mismatching lemma first tries the eight present-tense classes; if
one covers all present/subjunctive/imperative gold slots it goes to
data/ron/classes.tsv, otherwise the deviant slots go to
data/ron/verbs.tsv as an explicit row. Iterate with golden_ron until dry.
"""
import collections
import sys

MISMATCHES = "target/golden_ron_mismatches.tsv"
ORACLES = ["data/ron/dexonline.tsv", "data/ron/kaikki.tsv"]

PRS = [f"V;IND;PRS;{n};{p}" for n, p in
       [("SG", 1), ("SG", 2), ("SG", 3), ("PL", 1), ("PL", 2), ("PL", 3)]]


def load_gold():
    golds = []
    for path in ORACLES:
        g = collections.defaultdict(dict)
        for line in open(path):
            f = line.rstrip("\n").split("\t")
            if len(f) < 3:
                continue
            g[f[0]].setdefault(f[2], set()).add(f[1])
        golds.append(g)
    a, b = golds
    gold = collections.defaultdict(dict)
    for lemma, feats in a.items():
        if lemma not in b:
            # dexonline-only lemmas: mined from the one oracle, like
            # the SALDO-only Swedish compounds.
            for feat, va in feats.items():
                gold[lemma][feat] = (va, va)
            continue
        for feat, va in feats.items():
            vb = b[lemma].get(feat)
            if vb and va & vb:
                gold[lemma][feat] = (va | vb, va & vb)
    return gold


def palatalize(s):
    for a, b in [("st", "șt"), ("t", "ț"), ("d", "z"), ("s", "ș")]:
        if s.endswith(a):
            return s[: -len(a)] + b
    return s


def diphthongize(s):
    for i in range(len(s) - 1, -1, -1):
        if s[i] == "e":
            return s[:i] + "ea" + s[i + 1:]
        if s[i] == "o":
            return s[:i] + "oa" + s[i + 1:]
        if s[i] in "aăâîiu":
            return s
    return s


def theme(inf):
    if inf.endswith("ea"):
        return "ea"
    for t in ("a", "e", "i", "î"):
        if inf.endswith(t):
            return t
    return None


def stem(inf):
    return inf[: -2 if theme(inf) == "ea" else -1]


def aug_h(s):
    return s + "h" if s.endswith(("c", "g")) else s


def class_forms(inf, cls):
    """present 6 + sbjv3 + imp2 under a class, or None if inapplicable."""
    s = stem(inf)
    vowel = s.endswith(tuple("aeiou"))
    th = theme(inf)
    if cls == "a-ez":
        if th != "a":
            return None
        a = aug_h(s)
        p = [a + "ez", a + "ezi", a + "ează",
             s + ("em" if vowel else "ăm"), s + "ați", a + "ează"]
        return p + [a + "eze", p[2]]
    if cls == "a-bare":
        if th != "a":
            return None
        p = [s, palatalize(s) + "i", diphthongize(s) + "ă",
             s + ("em" if vowel else "ăm"), s + "ați", diphthongize(s) + "ă"]
        return p + [diphthongize(s) + "e", p[2]]
    if cls in ("e", "ea"):
        if th not in ("e", "ea"):
            return None
        p = [s, palatalize(s) + "i", s + "e", s + "em", s + "eți", s]
        return p + [diphthongize(s) + "ă", p[2]]
    if cls == "i-esc":
        if th != "i":
            return None
        p = [s + "esc", s + "ești", s + "ește", s + "im", s + "iți", s + "esc"]
        return p + [s + "ească", p[2]]
    if cls == "i-bare":
        if th != "i":
            return None
        p = [s, palatalize(s) + "i", diphthongize(s) + "e",
             s + "im", s + "iți", s]
        return p + [diphthongize(s) + "ă", p[2]]
    if cls == "ih-asc":
        if th != "î":
            return None
        p = [s + "ăsc", s + "ăști", s + "ăște", s + "âm", s + "âți", s + "ăsc"]
        return p + [s + "ască", p[2]]
    if cls == "ih-bare":
        if th != "î":
            return None
        p = [s, palatalize(s) + "i", diphthongize(s) + "ă",
             s + "âm", s + "âți", diphthongize(s) + "ă"]
        return p + [diphthongize(s) + "ă", p[2]]
    return None


def read_table(path):
    out = {}
    header = []
    for line in open(path):
        line = line.rstrip("\n")
        if line.startswith("#"):
            header.append(line)
            continue
        if line:
            out[line.split("\t")[0]] = line
    return header, out


def pick(variants, shared):
    """Canonical spelling: one both oracles attest, else any."""
    pool = sorted(shared) or sorted(variants)
    return pool[0]


def main():
    gold = load_gold()
    lemmas = sorted({l.split("\t")[0] for l in open(MISMATCHES) if l.strip()})
    ch, classes = read_table("data/ron/classes.tsv")
    vh, verbs = read_table("data/ron/verbs.tsv")
    n_class = n_row = 0
    for lemma in lemmas:
        feats = gold.get(lemma)
        if not feats or lemma in verbs:
            continue
        prs = [feats.get(k) for k in PRS]
        sb3 = feats.get("V;SBJV;PRS;SG;3")
        im2 = feats.get("V;IMP;SG;2")
        # A class must cover every attested present/sbjv3/imp2 slot.
        chosen = None
        for cls in ("a-ez", "a-bare", "e", "i-esc", "i-bare",
                    "ih-asc", "ih-bare"):
            cf = class_forms(lemma, cls)
            if cf is None:
                continue
            slots = list(zip(cf[:6], prs)) + [(cf[6], sb3), (cf[7], im2)]
            if all(g is None or f in g[0] for f, g in slots):
                chosen = cls
                break
        if chosen and lemma not in classes:
            classes[lemma] = f"{lemma}\t{chosen}"
            n_class += 1
            continue
        if chosen:
            continue
        # Explicit row from the gold slots.
        col = []
        for k in PRS:
            g = feats.get(k)
            col.append(pick(*g) if g else "-")
        for k in ("V;SBJV;PRS;SG;3", "V;IMP;SG;2", "V.PTCP;PST;MASC;SG",
                  "V;GER", "V;IND;PST;IPFV;SG;1", "V;IND;PST;PFV;SG;3"):
            g = feats.get(k)
            col.append(pick(*g) if g else "-")
        verbs[lemma] = lemma + "\t" + "\t".join(col)
        n_row += 1
    # Patch pass: lemmas that still mismatch on slots a class cannot
    # carry (strong participles, perfect stems) get the deviant
    # columns filled in their row, existing or fresh.
    COL = {"V;SBJV;PRS;SG;3": 7, "V;SBJV;PRS;PL;3": 7, "V;IMP;SG;2": 8,
           "V.PTCP;PST;MASC;SG": 9, "V;GER": 10}
    n_patch = 0
    for line in open(MISMATCHES):
        f = line.rstrip("\n").split("\t")
        if len(f) < 2:
            continue
        lemma, feat = f[0], f[1]
        feats = gold.get(lemma)
        if not feats:
            continue
        if feat in PRS:
            i = 1 + PRS.index(feat)
        elif feat in COL:
            i = COL[feat]
        elif ";PST;IPFV;" in feat:
            i, feat = 11, "V;IND;PST;IPFV;SG;1"
        elif ";PST;PFV;" in feat or ";PST;PQP;" in feat:
            i, feat = 12, "V;IND;PST;PFV;SG;3"
        else:
            continue
        g = feats.get(feat)
        if not g:
            continue
        row = verbs.get(lemma, lemma + "\t" + "\t".join(["-"] * 12))
        cols = row.split("\t")
        if cols[i] == "-":
            cols[i] = pick(*g)
            verbs[lemma] = "\t".join(cols)
            n_patch += 1
    with open("data/ron/classes.tsv", "w") as f:
        f.write("\n".join(ch + [classes[k] for k in sorted(classes)]) + "\n")
    with open("data/ron/verbs.tsv", "w") as f:
        f.write("\n".join(vh + [verbs[k] for k in sorted(verbs)]) + "\n")
    print(f"patched {n_patch}")
    print(f"classes +{n_class} (total {len(classes)}), "
          f"rows +{n_row} (total {len(verbs)})")


if __name__ == "__main__":
    sys.exit(main())
