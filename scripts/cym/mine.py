#!/usr/bin/env python3
"""Welsh productive literary-conjugation rules + mined exceptions.

Welsh is cited by its verbal noun (berfenw). The synthetic literary paradigm
is built from a STEM plus a fixed set of personal endings that are the same
for every regular verb (modelled on the perfectly regular "dysgu" -> dysg-):

  present    1sg -af  2sg -i   3sg -()  1pl -wn  2pl -wch 3pl -ant impers -ir
  imperfect  1sg -wn  2sg -it  3sg -ai  1pl -em  2pl -ech 3pl -ent impers -id
  preterite  1sg -ais 2sg -aist 3sg -odd 1pl -asom 2pl -asoch 3pl -asant  -wyd
  pluperf.   1sg -aswn 2sg -asit 3sg -asai 1pl -asem 2pl -asech 3pl -asent -asid
  subjunct.  1sg -wyf 2sg -ych  3sg -o   1pl -om  2pl -och  3pl -ont impers -er
  imperative 2sg -()  3sg -ed  1pl -wn  2pl -wch 3pl -ent impers -er
  V.MSDR = the verbal noun itself      V.PTCP = stem + -edig

The productive STEM is the verbal noun minus its final vowel (canu->can,
codi->cod, gweithio->gweithi); consonant-final verbal nouns keep their shape
(agor->agor). What the rules cannot predict is lexical and mined into
data/cym/parts.tsv: irregular stems (addo->addaw-, cael->caff-), the vowel
"affection" that fronts a stem vowel before -i / -ir endings (talu -> teli,
telir, telais) or lengthens the bare 3sg/imperative (tal -> tal), and the
suppletive verbs (bod, mynd, gwneud, dod, cael). Where the two oracles
(UniMorph and kaikki.org) attest the same form we store that; otherwise the
UniMorph reading.

Usage: mine.py data/cym/unimorph.tsv data/cym/kaikki.tsv > data/cym/parts.tsv
"""
import sys
from collections import defaultdict

CELLS = [
    "V;LIT;1;SG;IND;PRS", "V;LIT;2;SG;IND;PRS", "V;LIT;3;SG;IND;PRS",
    "V;LIT;1;PL;IND;PRS", "V;LIT;2;PL;IND;PRS", "V;LIT;3;PL;IND;PRS",
    "V;LIT;4;IND;PRS",
    "V;LIT;1;SG;IND;IPFV", "V;LIT;2;SG;IND;IPFV", "V;LIT;3;SG;IND;IPFV",
    "V;LIT;1;PL;IND;IPFV", "V;LIT;2;PL;IND;IPFV", "V;LIT;3;PL;IND;IPFV",
    "V;LIT;4;IND;IPFV",
    "V;LIT;1;SG;IND;PST", "V;LIT;2;SG;IND;PST", "V;LIT;3;SG;IND;PST",
    "V;LIT;1;PL;IND;PST", "V;LIT;2;PL;IND;PST", "V;LIT;3;PL;IND;PST",
    "V;LIT;4;IND;PST",
    "V;LIT;1;SG;IND;PST;PFV", "V;LIT;2;SG;IND;PST;PFV", "V;LIT;3;SG;IND;PST;PFV",
    "V;LIT;1;PL;IND;PST;PFV", "V;LIT;2;PL;IND;PST;PFV", "V;LIT;3;PL;IND;PST;PFV",
    "V;LIT;4;IND;PST;PFV",
    "V;LIT;1;SG;SBJV", "V;LIT;2;SG;SBJV", "V;LIT;3;SG;SBJV",
    "V;LIT;1;PL;SBJV", "V;LIT;2;PL;SBJV", "V;LIT;3;PL;SBJV",
    "V;LIT;4;SBJV",
    "V;LIT;2;SG;IMP", "V;LIT;3;SG;IMP", "V;LIT;1;PL;IMP",
    "V;LIT;2;PL;IMP", "V;LIT;3;PL;IMP", "V;LIT;4;IMP",
    "V;V.MSDR", "V;V.PTCP",
]

ENDINGS = {
    "prs": {"1sg": "af", "2sg": "i", "3sg": "", "1pl": "wn", "2pl": "wch",
            "3pl": "ant", "4": "ir"},
    "ipf": {"1sg": "wn", "2sg": "it", "3sg": "ai", "1pl": "em", "2pl": "ech",
            "3pl": "ent", "4": "id"},
    "pst": {"1sg": "ais", "2sg": "aist", "3sg": "odd", "1pl": "asom",
            "2pl": "asoch", "3pl": "asant", "4": "wyd"},
    "ppf": {"1sg": "aswn", "2sg": "asit", "3sg": "asai", "1pl": "asem",
            "2pl": "asech", "3pl": "asent", "4": "asid"},
    "sbjv": {"1sg": "wyf", "2sg": "ych", "3sg": "o", "1pl": "om", "2pl": "och",
             "3pl": "ont", "4": "er"},
    "imp": {"2sg": "", "3sg": "ed", "1pl": "wn", "2pl": "wch", "3pl": "ent",
            "4": "er"},
}

VOWELS = "aeiouwyâêîôûŵŷàèìòùáéíóúäëïöü"


def stem(cit):
    """Productive stem: the verbal noun minus a final vowel."""
    if cit and cit[-1] in VOWELS:
        return cit[:-1]
    return cit


def tensemood(feat):
    if feat.endswith("IMP"):
        return "imp"
    if feat.endswith("SBJV"):
        return "sbjv"
    if "IND;PST;PFV" in feat:
        return "ppf"
    if feat.endswith("IND;PST"):
        return "pst"
    if feat.endswith("IND;IPFV"):
        return "ipf"
    if feat.endswith("IND;PRS"):
        return "prs"
    return None


def personkey(parts):
    if parts[2] == "4":
        return "4"
    return parts[2] + ("sg" if parts[3] == "SG" else "pl")


def productive(cit, feat):
    """The rule-generated form, or None if the paradigm has no such slot."""
    if feat == "V;V.MSDR":
        return cit
    st = stem(cit)
    if feat == "V;V.PTCP":
        return st + "edig"
    if not feat.startswith("V;LIT;"):
        return None
    tm = tensemood(feat)
    if tm is None:
        return None
    end = ENDINGS[tm].get(personkey(feat.split(";")))
    if end is None:
        return None
    return st + end


def main(uni_path, kai_path):
    def read(path):
        d = defaultdict(set)
        for line in open(path):
            a = line.rstrip("\n").split("\t")
            if len(a) >= 3 and a[2].startswith("V"):
                d[(a[0], a[2])].add(a[1])
        return d
    uni, kai = read(uni_path), read(kai_path)

    def chosen(lemma, feat):
        a, b = uni.get((lemma, feat), set()), kai.get((lemma, feat), set())
        pool = (a & b) or a or b
        return sorted(pool)[0] if pool else None

    lemmas = sorted({l for (l, _f) in list(uni) + list(kai)})

    total = hit = 0
    parts = {}
    for lemma in lemmas:
        if not lemma or " " in lemma:
            continue
        row = {}
        for c in CELLS:
            f = chosen(lemma, c)
            if f is None:
                row[c] = "-"
                continue
            pred = productive(lemma, c)
            if pred is not None:
                total += 1
                if pred == f or (uni.get((lemma, c)) and pred in uni[(lemma, c)]) \
                        or (kai.get((lemma, c)) and pred in kai[(lemma, c)]):
                    hit += 1
            row[c] = f if f != pred else "-"
        if any(v != "-" for v in row.values()):
            parts[lemma] = row

    sys.stderr.write(
        f"rule accuracy: {hit}/{total} = {100 * hit / max(total, 1):.2f}%, "
        f"mined lemmas: {len(parts)}\n")
    print("# lemma(verbal-noun)\t" + "\t".join(CELLS) +
          '  ("-" = productive default)')
    for lemma in sorted(parts):
        print(lemma + "\t" + "\t".join(parts[lemma].get(c, "-") for c in CELLS))


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
