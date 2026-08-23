#!/usr/bin/env python3
"""Mine the Macedonian exception table from the two-oracle AGREEMENT.

Reads apertium.tsv and unimorph.tsv (both `lemma <TAB> form <TAB>
features`), keeps only the (lemma, features) slots where the two oracles
share a form, and emits `data/mkd/verbs.tsv` — a row wherever a covered
slot's agreed form is not what the productive rule in `src/mkd.rs`
generates. Mining from the agreement (not UniMorph alone) keeps UniMorph
data errors — e.g. the lemma `автоматизира` whose paradigm is mis-entered
as `идеализира…` — out of the engine: apertium disagrees there, so the
slot is excluded.

Macedonian has no infinitive; the lemma is the 3sg present, from whose
final vowel the class is read (-а a, -и i, -е e). Covered slots: present,
imperfect (V;PROG;PST), the imperfect l-participle, imperative, the
passive participle and the converb/verbal-noun. Columns:

  lemma  prs1sg prs2sg prs3sg prs1pl prs2pl prs3pl
         impf1sg impf2sg impf3sg impf1pl impf2pl impf3pl
         lp_m lp_f lp_n lp_pl imp2sg imp2pl pass converb vnoun

`-` = the rule; `|` separates variants.

Usage: python3 scripts/mkd/mine.py data/mkd/apertium.tsv data/mkd/unimorph.tsv
"""
import collections
import sys

SLOTS = [
    "V;PRS;1;SG", "V;PRS;2;SG", "V;PRS;3;SG", "V;PRS;1;PL", "V;PRS;2;PL", "V;PRS;3;PL",
    "V;PROG;PST;1;SG", "V;PROG;PST;2;SG", "V;PROG;PST;3;SG",
    "V;PROG;PST;1;PL", "V;PROG;PST;2;PL", "V;PROG;PST;3;PL",
    "V.PTCP;PROG;PST;MASC;SG", "V.PTCP;PROG;PST;FEM;SG",
    "V.PTCP;PROG;PST;NEUT;SG", "V.PTCP;PROG;PST;PL",
    "V;IMP;2;SG", "V;IMP;2;PL", "V.PTCP;PST;PASS", "V.CVB", "V.MSDR",
]


def cls(lem):
    if lem.endswith("а"):
        return "a", lem[:-1]
    if lem.endswith("и"):
        return "i", lem[:-1]
    if lem.endswith("е"):
        return "e", lem[:-1]
    return "?", lem


def rules(lem):
    c, s = cls(lem)
    if c == "?":
        return {}
    th = "а" if c == "a" else "е"  # imperfect / l-participle theme
    return {
        "V;PRS;1;SG": s + "ам", "V;PRS;3;SG": s + ("а" if c == "a" else ("и" if c == "i" else "е")),
        "V;PRS;2;SG": s + ("аш" if c == "a" else ("иш" if c == "i" else "еш")),
        "V;PRS;1;PL": s + ("аме" if c == "a" else ("име" if c == "i" else "еме")),
        "V;PRS;2;PL": s + ("ате" if c == "a" else ("ите" if c == "i" else "ете")),
        "V;PRS;3;PL": s + ("аат" if c == "a" else "ат"),
        "V;PROG;PST;1;SG": s + th + "в", "V;PROG;PST;2;SG": s + th + "ше", "V;PROG;PST;3;SG": s + th + "ше",
        "V;PROG;PST;1;PL": s + th + "вме", "V;PROG;PST;2;PL": s + th + "вте", "V;PROG;PST;3;PL": s + th + "а",
        "V.PTCP;PROG;PST;MASC;SG": s + th + "л", "V.PTCP;PROG;PST;FEM;SG": s + th + "ла",
        "V.PTCP;PROG;PST;NEUT;SG": s + th + "ло", "V.PTCP;PROG;PST;PL": s + th + "ле",
        "V;IMP;2;SG": s + ("ај" if c == "a" else "и"), "V;IMP;2;PL": s + ("ајте" if c == "a" else "ете"),
        "V.PTCP;PST;PASS": s + ("ан" if c == "a" else "ен"),
        "V.CVB": s + ("ајќи" if c == "a" else "ејќи"), "V.MSDR": s + ("ање" if c == "a" else "ење"),
    }


def load(path):
    g = collections.defaultdict(lambda: collections.defaultdict(set))
    for line in open(path, encoding="utf-8"):
        p = line.rstrip("\n").split("\t")
        if len(p) < 3 or not p[2].startswith("V"):
            continue
        g[p[0]][p[2]].add(p[1])
    return g


VOWELS = "аеиоу"


def vowel_final_stem(lem):
    """A j-stem: the lemma's stem (minus the thematic vowel) ends in a
    vowel (брои → бро, пее → пе). apertium over-regularizes these
    (броам for бројам), so for them UniMorph is the adjudicated oracle."""
    c, st = cls(lem)
    return c != "?" and st and st[-1] in VOWELS


def main(apertium, unimorph):
    a, u = load(apertium), load(unimorph)
    out = []
    for lem in sorted(set(a) | set(u)):
        r = rules(lem)
        jstem = vowel_final_stem(lem)
        cells = []
        dev = False
        for slot in SLOTS:
            if jstem and slot not in ("V.CVB", "V.MSDR"):
                # Trust UniMorph for the j-stems (apertium is wrong here) —
                # except the converb/verbal-noun, where the rule is correct and
                # UniMorph carries the ќ→к converb typo.
                uv = u.get(lem, {}).get(slot)
                if not uv:
                    cells.append("-")
                    continue
                if r.get(slot) in uv:
                    cells.append("-")
                else:
                    cells.append("|".join(sorted(uv)))
                    dev = True
                continue
            av, uv = a.get(lem, {}).get(slot), u.get(lem, {}).get(slot)
            # agreement: both present and overlapping (or one present when the
            # other oracle lacks the slot entirely — converb/vnoun apertium-only
            # or unimorph-only are still trustworthy singletons).
            # Require genuine agreement: both oracles present and overlapping.
            # A slot only one oracle has (or where they disagree) is left to
            # the rule and excluded from the gold — this is what keeps
            # UniMorph-only errors (автоматизира→идеализира…) out of the engine.
            if not (av and uv):
                cells.append("-")
                continue
            agreed = av & uv
            if not agreed:
                cells.append("-")
                continue
            if r.get(slot) in agreed:
                cells.append("-")
            else:
                cells.append("|".join(sorted(agreed)))
                dev = True
        if dev:
            out.append("\t".join([lem] + cells))
    hdr = ("# Macedonian mined principal parts, from the apertium-mkd ∩ UniMorph "
           "agreement (stress stripped). Columns: lemma, prs1sg..3pl, impf1sg..3pl, "
           "lp_masc/fem/neut/pl, imp2sg, imp2pl, pass, converb, vnoun.\n"
           "# '-' = the productive rule in src/mkd.rs; '|' separates variants.\n")
    sys.stdout.write(hdr + "\n".join(out) + "\n")


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
