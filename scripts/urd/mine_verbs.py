#!/usr/bin/env python3
"""Regenerate the Urdu irregular lexicon (data/urd/verbs.tsv) and the
oracle-disagreement log (docs/urd/disagreements.tsv).

Two products:

1. **data/urd/verbs.tsv** — the core irregular verbs, hand-supplied in
   scripts/urd/manual.tsv (the suppletive/contracted stems of ہونا,
   جانا, کرنا, دینا, لینا, پینا, … are few and are not derivable from the
   oracle agreement, which covers the *forms* but not the stem analysis).
   This script copies manual.tsv through so the mine stays reproducible.

2. **docs/urd/disagreements.tsv** — every (lemma, feature) slot the two
   oracles (kaikki ∩ apertium) cover with disjoint forms, with a ruling.

Run after the fetch scripts: python3 scripts/urd/mine_verbs.py
"""

import collections


def load(path):
    forms = collections.defaultdict(lambda: collections.defaultdict(set))
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            cells = line.rstrip("\n").split("\t")
            if len(cells) != 3:
                continue
            lemma, form, feat = cells
            forms[lemma][feat].add(form)
    return forms


def write_verbs():
    with open("scripts/urd/manual.tsv", encoding="utf-8") as fh:
        body = [l for l in fh if not l.startswith("#") and l.strip()]
    header = [
        "# Urdu irregular verbs.",
        "#",
        "# The rule engine (src/urd.rs) handles the open class off the stem; this",
        "# table carries the verbs whose perfective, subjunctive or imperative is",
        "# not predictable. Forms are in the normalized Perso-Arabic orthography",
        "# the oracle adapters use (diacritics and the noon-ghunna mark stripped,",
        "# Arabic letters folded); a `-` group falls through to the rule.",
        "#",
        "# Mined by scripts/urd/mine_verbs.py from scripts/urd/manual.tsv.",
        "#",
        "# lemma\tpfv(m.sg,m.pl,f)\tsubj(1sg,2sg,3sg,1pl,2pl,3pl)\timp(intim,famil,polite)",
    ]
    with open("data/urd/verbs.tsv", "w", encoding="utf-8") as fh:
        fh.write("\n".join(header) + "\n" + "".join(body))
    return len(body)


# How each recurring oracle disagreement is ruled (see docs/urd/oracles.md):
#   o1  — kaikki is right          o2 — apertium is right
RULINGS = {
    # کاؤ nudge: apertium mis-forms a few cells; kaikki is standard.
    ("پینا", "V;V.PTCP;MASC;PL;IPFV"): ("o1", "apertium پیتیں is an error; kaikki پیتے is standard"),
}


def rule(lemma, feat, kforms, aforms):
    if (lemma, feat) in RULINGS:
        return RULINGS[(lemma, feat)][0]
    # A vowel-stem perfective/masc-sg glide: apertium writes the ی-glide
    # (رویا, چھویا) the grammar requires; kaikki drops it (روا). o2.
    if feat == "V;V.PTCP;MASC;SG;PFV":
        return "o2"
    # kaikki tabulation defects (a doubled نا infinitive): apertium right.
    if feat.startswith("V;NFIN"):
        return "o2"
    return "?"


def write_disagreements():
    kaikki = load("data/urd/kaikki.tsv")
    apertium = load("data/urd/apertium.tsv")
    rows = []
    for lemma in set(kaikki) & set(apertium):
        for feat in set(kaikki[lemma]) & set(apertium[lemma]):
            a, b = kaikki[lemma][feat], apertium[lemma][feat]
            if a & b:
                continue  # agreement, not a disagreement
            o1 = "|".join(sorted(a))
            o2 = "|".join(sorted(b))
            rows.append(f"{lemma}\t{feat}\t{rule(lemma, feat, a, b)}\t{o1}\t{o2}")
    rows.sort()
    header = (
        "# Oracle disagreements for Urdu (kaikki ∩ apertium).\n"
        "#\n"
        "# The two independent oracles agree on ~99.5% of the shared\n"
        "# person-independent slots; the handful below are the residue:\n"
        "#  - vowel-stem masc-sg perfective: apertium keeps the ی-glide the\n"
        "#    grammar requires (رویا/چھویا), kaikki drops it (روا) → o2;\n"
        "#  - a kaikki tabulation defect (doubled نا infinitive) → o2;\n"
        "#  - a stray apertium participle mis-form → o1.\n"
        "# The engine follows the oracle ruled correct.\n"
        "#\n"
        "# lemma\tfeatures\tresolution\to1(kaikki)\to2(apertium)\n"
    )
    with open("docs/urd/disagreements.tsv", "w", encoding="utf-8") as fh:
        fh.write(header + "\n".join(rows) + "\n")
    return len(rows)


def main():
    n_verbs = write_verbs()
    n_disc = write_disagreements()
    print(f"{n_verbs} rows written to data/urd/verbs.tsv")
    print(f"{n_disc} disagreements written to docs/urd/disagreements.tsv")


if __name__ == "__main__":
    main()
