#!/usr/bin/env python3
"""Mine the Tamil verb stem table (data/tam/verbs.tsv).

A Tamil verb's conjugation class is lexical — the root does not tell you
its past marker (செய்த், வந்த், ஓடின், படித்த்) or whether it is strong
or weak — so, like a German strong verb's principal parts, the stems are
stored and the engine (src/tam.rs) only stacks the person-number-gender
endings on top. This mines those stems from the two-oracle agreement
(kaikki ∩ ThamizhiMorph): the present, past and future person stems are
read off the 3sg-masculine cell (strip -ஆன், restore the pulli), and the
-உம் form, the infinitive and the plural imperative are taken whole.

A stem is taken from the slot where both oracles agree; where they
disagree (a lexical strong/weak or class split, e.g. படி as both
படித்த் and படிந்த்) the slot is not scored anyway, and kaikki — the
literary reference — is used so the row stays internally consistent.

Regenerate: python3 scripts/tam/mine_verbs.py
"""
import collections

# The 3sg-masculine ending as it surfaces after a pulli stem: ா ன ்
AAN_SURFACE = "ான்"
PULLI = "்"


def load(path):
    g = collections.defaultdict(lambda: collections.defaultdict(set))
    for line in open(path, encoding="utf-8"):
        f = line.rstrip("\n").split("\t")
        if len(f) != 3:
            continue
        g[f[0]][f[2]].add(f[1])
    return g


KAIKKI = load("data/tam/kaikki.tsv")
THAMIZHI = load("data/tam/thamizhi.tsv")


def pick(lemma, feat):
    """The agreed form for a slot, else kaikki's; None if kaikki lacks it."""
    a = KAIKKI.get(lemma, {}).get(feat, set())
    b = THAMIZHI.get(lemma, {}).get(feat, set())
    both = a & b
    if both:
        return sorted(both)[0]
    return sorted(a)[0] if a else None


def stem_from_3sgm(lemma, feat):
    """Recover a tense stem: strip the surfaced -ஆன் and restore the pulli."""
    form = pick(lemma, feat)
    if form and form.endswith(AAN_SURFACE):
        return form[: -len(AAN_SURFACE)] + PULLI
    return None


def main():
    lemmas = sorted(set(KAIKKI) & set(THAMIZHI))
    rows = []
    for lemma in lemmas:
        present = stem_from_3sgm(lemma, "V;PRS;3SGM")
        past = stem_from_3sgm(lemma, "V;PST;3SGM")
        future = stem_from_3sgm(lemma, "V;FUT;3SGM")
        um = pick(lemma, "V;FUT;3SGN") or pick(lemma, "V;PTCP;FUT")
        inf = pick(lemma, "V;INF")
        imp_pl = pick(lemma, "V;IMP;PL")
        cols = [present, past, future, um, inf, imp_pl]
        if not any(cols):
            continue
        rows.append([lemma] + [c if c else "-" for c in cols])

    header = [
        "# Tamil verb stems, mined from the kaikki ∩ ThamizhiMorph agreement.",
        "# Tamil conjugation class is lexical, so the stems are stored and the",
        "# engine (src/tam.rs) only stacks the PNG endings.",
        "# Regenerate: python3 scripts/tam/mine_verbs.py",
        "#",
        "# present is the கிற் stem, past the past stem, future the வ்/ப் stem;",
        "# um is the -உம் form, inf the infinitive, imp_pl the plural imperative.",
        "# A `-` cell falls through to the productive weak rule.",
        "# lemma\tpresent\tpast\tfuture\tum\tinf\timp_pl",
    ]
    with open("data/tam/verbs.tsv", "w", encoding="utf-8") as fh:
        fh.write("\n".join(header + ["\t".join(r) for r in rows]) + "\n")
    print(f"{len(rows)} rows written to data/tam/verbs.tsv")


if __name__ == "__main__":
    main()
