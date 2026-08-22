#!/usr/bin/env python3
"""Regenerate the Hindi irregular lexicon (data/hin/verbs.tsv) and the
oracle-disagreement log (docs/hin/disagreements.tsv).

Two products:

1. **data/hin/verbs.tsv** — the core irregular verbs. Unlike the other
   languages, these are hand-supplied in scripts/hin/manual.tsv (the
   suppletive/contracted stems of होना, जाना, करना, देना, लेना, पीना,
   … are few and are not derivable from the oracle agreement, which
   covers the *forms* but not the stem analysis). This script just
   copies manual.tsv through, so the mine stays reproducible.

2. **docs/hin/disagreements.tsv** — every (lemma, feature) slot the two
   oracles cover with disjoint forms. In Hindi these are all one
   systematic UniMorph defect: an independent vowel spliced onto a
   consonant stem (बनऊं, बनया, बनइए) where the matra is required (बनूं,
   बना, बनिए). kaikki (oracle 1) is correct, so each is ruled o1.

Run after the fetch scripts: python3 scripts/hin/mine_verbs.py
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
    with open("scripts/hin/manual.tsv", encoding="utf-8") as fh:
        body = [l for l in fh if not l.startswith("#") and l.strip()]
    header = [
        "# Hindi irregular verbs.",
        "#",
        "# The rule engine (src/hin.rs) handles the open class off the stem; this",
        "# table carries the verbs whose perfective, subjunctive or imperative is",
        "# not predictable. Forms are in the normalized orthography the oracle",
        "# adapters use (anusvara for candrabindu, no optional य-glide, no",
        "# feminine-plural nasal); a `-` group falls through to the rule.",
        "#",
        "# Mined by scripts/hin/mine_verbs.py from scripts/hin/manual.tsv.",
        "#",
        "# lemma\tpfv(m.sg,m.pl,f)\tsubj(1sg,2sg,3sg,1pl,2pl,3pl)\timp(intim,famil,polite)",
    ]
    with open("data/hin/verbs.tsv", "w", encoding="utf-8") as fh:
        fh.write("\n".join(header) + "\n" + "".join(body))
    return len(body)


def write_disagreements():
    kaikki = load("data/hin/kaikki.tsv")
    unimorph = load("data/hin/unimorph.tsv")
    rows = []
    for lemma in set(kaikki) & set(unimorph):
        for feat in set(kaikki[lemma]) & set(unimorph[lemma]):
            a, b = kaikki[lemma][feat], unimorph[lemma][feat]
            if a & b:
                continue  # agreement, not a disagreement
            o1 = "|".join(sorted(a))
            o2 = "|".join(sorted(b))
            rows.append(f"{lemma}\t{feat}\to1\t{o1}\t{o2}")
    rows.sort()
    header = (
        "# Oracle disagreements for Hindi (kaikki ∩ UniMorph).\n"
        "#\n"
        "# Every row here is one systematic UniMorph hin defect: for a subset\n"
        "# of lemmas it forms the perfective participle and the vowel-initial\n"
        "# subjunctive/imperative endings with an independent vowel spliced\n"
        "# onto the consonant stem (बनऊं, बनया, बनइए) instead of the matra the\n"
        "# grammar requires (बनूं, बना, बनिए). kaikki.org spells them\n"
        "# correctly, so every disagreement is ruled o1 (first oracle = kaikki).\n"
        "# The engine follows kaikki.\n"
        "#\n"
        "# lemma\tfeatures\tresolution\to1(kaikki)\to2(unimorph)\n"
    )
    with open("docs/hin/disagreements.tsv", "w", encoding="utf-8") as fh:
        fh.write(header + "\n".join(rows) + "\n")
    return len(rows)


def main():
    n_verbs = write_verbs()
    n_disc = write_disagreements()
    print(f"{n_verbs} rows written to data/hin/verbs.tsv")
    print(f"{n_disc} disagreements written to docs/hin/disagreements.tsv")


if __name__ == "__main__":
    main()
