#!/usr/bin/env python3
"""Mine the Turkish exception table (data/tur/verbs.tsv).

Turkish is highly regular, so the table is small: the closed class of
monosyllables whose aorist is -Ir rather than the default -Ar (al → alır,
gel → gelir, az → azır), the verbs that voice a final t→d before a vowel
(git → gider, keşfet → keşfeder) and the suppletive stems of demek/yemek
(diyor, diyecek, diyin). Everything else derives.

Each stored cell is a 3rd-singular predicative base (or the imperative);
the personal endings still derive by rule. A cell is mined only where the
two oracles *intersect on a single form*, so the buggy UniMorph
necessitative (yazmeli for yazmalı, which kaikki spells correctly) and the
potential-form pollution in kaikki's imperative both drop out on their own.

The input is scripts/tur/irregulars.txt — the lemmas the productive rules
miss against the agreement, captured with an empty table by
scripts/tur/capture_irregulars.sh, so the mine is reproducible.

Run after a golden_tur pass:
    ./scripts/tur/capture_irregulars.sh
    python3 scripts/tur/mine_verbs.py
"""

import collections

COLUMNS = ["aorist", "progressive", "future", "past", "evidential",
           "necessitative", "imp_sg", "imp_pl"]
# The 3sg (or imperative) slot each column is read from.
SLOTS = {
    "aorist": "V;IND;PRS;HAB;3;SG;POS",
    "progressive": "V;IND;PRS;PROG;3;SG;POS",
    "future": "V;IND;FUT;3;SG;POS",
    "past": "V;IND;PST;3;SG;POS",
    "evidential": "V;INFR;PST;3;SG;POS",
    "necessitative": "V;OBLIG;PRS;3;SG;POS",
    "imp_sg": "V;IMP;2;SG;POS",
    "imp_pl": "V;IMP;2;PL;POS",
}


def load(path):
    forms = collections.defaultdict(lambda: collections.defaultdict(set))
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            f = line.rstrip("\n").split("\t")
            if len(f) == 3:
                forms[f[0]][f[2]].add(f[1])
    return forms


UNIMORPH = load("data/tur/unimorph.tsv")
KAIKKI = load("data/tur/kaikki.tsv")


def agreed(lemma, feat):
    """The single form both oracles carry for a slot, or None."""
    both = UNIMORPH.get(lemma, {}).get(feat, set()) & KAIKKI.get(lemma, {}).get(feat, set())
    return sorted(both)[0] if len(both) == 1 else None


def mine(lemma):
    row = {c: "-" for c in COLUMNS}
    for col, slot in SLOTS.items():
        form = agreed(lemma, slot)
        if form:
            row[col] = form
    return row


def read_manual():
    rows = {}
    with open("scripts/tur/manual.tsv", encoding="utf-8") as fh:
        for line in fh:
            if line.startswith("#") or not line.strip():
                continue
            cells = line.rstrip("\n").split("\t")
            rows[cells[0]] = dict(zip(COLUMNS, cells[1:]))
    return rows


def main():
    irregulars = set()
    with open("scripts/tur/irregulars.txt", encoding="utf-8") as fh:
        for line in fh:
            line = line.strip()
            if line and not line.startswith("#"):
                irregulars.add(line)

    header = [
        "# Turkish irregular verbs — the stems the productive rules cannot",
        "# derive.",
        "#",
        "# Columns (a `-` falls through to the rule):",
        "#   lemma  aorist  progressive  future  past  evidential"
        "  necessitative  imp_sg  imp_pl",
        "# The stored cells are 3rd-singular predicative bases; personal",
        "# endings still derive by rule.",
        "#",
        "# Mined from the kaikki ∩ UniMorph agreement for the lemmas the",
        "# rules miss (empty-table capture, scripts/tur/capture_irregulars.sh).",
        "# Regenerate: python3 scripts/tur/mine_verbs.py",
        "#",
        "# lemma\t" + "\t".join(COLUMNS),
    ]
    rows = {lemma: mine(lemma) for lemma in irregulars}
    # The hand-supplied core wins over anything mined for the same lemma.
    rows.update(read_manual())
    body = []
    for lemma in sorted(rows):
        row = rows[lemma]
        if all(v == "-" for v in row.values()):
            continue
        body.append(lemma + "\t" + "\t".join(row[c] for c in COLUMNS))
    with open("data/tur/verbs.tsv", "w", encoding="utf-8") as fh:
        fh.write("\n".join(header + body) + "\n")
    print(f"{len(body)} rows written to data/tur/verbs.tsv")


if __name__ == "__main__":
    main()
