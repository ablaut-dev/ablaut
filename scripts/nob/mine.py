#!/usr/bin/env python3
"""Mine the Norwegian Bokmål exception table from a gold TSV.

Reads `lemma <TAB> form <TAB> features` (the two-oracle agreement in CI,
or UniMorph alone locally) and emits `data/nob/verbs.tsv`:

    lemma  present  past  past_participle  imperative  present_participle  present_passive

A cell is `-` when the productive default already yields a gold form, and
otherwise the `|`-joined gold variants. The default rules mirror
`src/nob.rs` exactly (class-1 -et/-a is the productive past); class-2
(-te/-de) and strong verbs, deponents and contracted verbs land here as
mined principal parts. Run after a fetch script:

    python3 scripts/nob/mine.py data/nob/unimorph.tsv > data/nob/verbs.tsv
"""
import collections
import sys

VOW = "aeiouyæøå"
SLOTS = [
    ("V;IND;PRS", "present"),
    ("V;IND;PST", "past"),
    ("V.PTCP;PST", "past_participle"),
    ("V;IMP", "imperative"),
    ("V.PTCP;PRS", "present_participle"),
    ("V;IND;PASS", "present_passive"),
]


def stem(inf):
    if inf.endswith("e") and len(inf) > 1 and inf[-2] not in VOW:
        return inf[:-1]
    return inf


def deponent(inf):
    return inf.endswith("s")


def default(inf, slot):
    s = stem(inf)
    if slot == "V;IND;PRS":
        return [inf] if deponent(inf) else [inf + "r"]
    if slot in ("V;IND;PST", "V.PTCP;PST"):
        return [s + "et", s + "a"]
    if slot == "V;IMP":
        return [s]
    if slot == "V.PTCP;PRS":
        return [inf + "nde"] if inf.endswith("e") else [inf + "ende"]
    if slot == "V;IND;PASS":
        return [inf] if deponent(inf) else [inf + "s"]
    return []


def main(path):
    rows = collections.defaultdict(lambda: collections.defaultdict(set))
    for line in open(path, encoding="utf-8"):
        p = line.rstrip("\n").split("\t")
        if len(p) < 3 or not p[2].startswith("V"):
            continue
        rows[p[0]][p[2]].add(p[1])
    out = []
    for lemma in sorted(rows):
        feats = rows[lemma]
        cells = []
        deviates = False
        for slot, _name in SLOTS:
            gold = feats.get(slot)
            if not gold:
                cells.append("-")
                continue
            if set(default(lemma, slot)) & gold:
                cells.append("-")
            else:
                cells.append("|".join(sorted(gold)))
                deviates = True
        if deviates:
            out.append("\t".join([lemma] + cells))
    sys.stdout.write(
        "# Norwegian Bokmål mined principal parts. Columns: lemma, present, "
        "past, past_participle, imperative, present_participle, present_passive.\n"
        "# '-' = the productive default in src/nob.rs; '|' separates variants.\n"
    )
    sys.stdout.write("\n".join(out) + "\n")


if __name__ == "__main__":
    main(sys.argv[1])
