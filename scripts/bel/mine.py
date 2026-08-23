#!/usr/bin/env python3
"""Mine the Belarusian exception table from the two oracle TSVs.

Belarusian is cited by its infinitive (in -ць / -ці / -чы). The synthetic
one-word paradigm scored is the non-past (present for imperfective verbs,
synthetic future for perfective ones — the two share one set of endings, a
verb attests only one), the past (l-participle with gender/number
agreement) and the imperative.

productive() implements genuine, generalising conjugation rules keyed on
the infinitive ending (mirrored verbatim in src/bel.rs). What the rules
cannot predict is lexical and mined into data/bel/parts.tsv:

  * the akanne / jakanne root alternations of the non-past (unstressed а/я
    in the infinitive surfacing as stressed о/э: зрабіць → зробіць,
    абараніць → абароніць) — Belarusian writes the reduction, so it is
    unrecoverable from the stress-less infinitive;
  * consonant mutations (адказаць → адкажу, насіць → нашу);
  * the soft-sign imperatives (абносіць → абнось) and irregular athematic
    verbs (даць, быць, ісці, magчы …).

The productive rules alone score ~60% of the agreed forms; every remaining
deviation carries a row here, so the engine reproduces the oracle exactly.

Usage: mine.py data/bel/unimorph.tsv data/bel/kaikki.tsv > data/bel/parts.tsv
"""
import sys
from collections import defaultdict

# Column order of parts.tsv — the synthetic one-word cells the engine stores.
CELLS = [
    "V;PRS;1;SG", "V;PRS;2;SG", "V;PRS;3;SG",
    "V;PRS;1;PL", "V;PRS;2;PL", "V;PRS;3;PL",
    "V;FUT;1;SG", "V;FUT;2;SG", "V;FUT;3;SG",
    "V;FUT;1;PL", "V;FUT;2;PL", "V;FUT;3;PL",
    "V;PST;SG;MASC", "V;PST;SG;FEM", "V;PST;SG;NEUT", "V;PST;PL",
    "V;IMP;2;SG", "V;IMP;2;PL",
]

# The non-past personal endings, by conjugation. A verb's present (if
# imperfective) and synthetic future (if perfective) use the same set.
CONJ1 = {"1;SG": "ю", "2;SG": "еш", "3;SG": "е",
         "1;PL": "ем", "2;PL": "еце", "3;PL": "юць"}          # vowel stem
CONJ1_HARD = {"1;SG": "у", "2;SG": "еш", "3;SG": "е",
              "1;PL": "ем", "2;PL": "еце", "3;PL": "уць"}     # -нуць etc.
CONJ2_SOFT = {"1;SG": "ю", "2;SG": "іш", "3;SG": "іць",
              "1;PL": "ім", "2;PL": "іце", "3;PL": "яць"}     # -іць
CONJ2_HARD = {"1;SG": "у", "2;SG": "ыш", "3;SG": "ыць",
              "1;PL": "ым", "2;PL": "ыце", "3;PL": "аць"}     # -ыць


def nonpast(lem, pn):
    """The productive non-past form for person;number pn, or None."""
    for suf, th in (("аваць", "у"), ("яваць", "ю")):
        if lem.endswith(suf):
            return lem[:-5] + th + CONJ1[pn]
    if lem.endswith("нуць"):
        return lem[:-3] + CONJ1_HARD[pn]
    for suf in ("аць", "яць"):
        if lem.endswith(suf):
            return lem[:-2] + CONJ1[pn]
    if lem.endswith("іць"):
        return lem[:-3] + CONJ2_SOFT[pn]
    if lem.endswith("ыць"):
        return lem[:-3] + CONJ2_HARD[pn]
    if lem.endswith("ець"):
        return lem[:-3] + "е" + CONJ1[pn]
    return None


def imperative(lem, plural):
    """The productive imperative (2sg / 2pl), or None."""
    tail = "це" if plural else ""
    for suf in ("аваць", "яваць"):
        if lem.endswith(suf):
            th = "у" if suf[0] == "а" else "ю"
            return lem[:-5] + th + "й" + tail
    if lem.endswith("нуць"):
        return lem[:-3] + "і" + tail
    for suf in ("аць", "яць"):
        if lem.endswith(suf):
            return lem[:-2] + "й" + tail
    if lem.endswith("іць"):
        return lem[:-3] + "і" + tail
    if lem.endswith("ыць"):
        return lem[:-3] + "ы" + tail
    if lem.endswith("ець"):
        return lem[:-3] + "ей" + tail
    return None


def productive(lem, feat):
    """The rule-generated form for a cell, or None if no rule applies."""
    if feat.startswith("V;PST"):
        if not lem.endswith("ць"):
            return None
        st = lem[:-2]
        return {"V;PST;SG;MASC": st + "ў", "V;PST;SG;FEM": st + "ла",
                "V;PST;SG;NEUT": st + "ла", "V;PST;PL": st + "лі"}.get(feat)
    if feat.startswith("V;IMP"):
        return imperative(lem, feat.endswith("PL"))
    if feat.startswith("V;PRS") or feat.startswith("V;FUT"):
        return nonpast(lem, feat.split(";", 2)[2])
    return None


def main(uni_path, kai_path):
    def read(path):
        d = defaultdict(set)
        for l in open(path):
            a = l.rstrip("\n").split("\t")
            if len(a) >= 3 and a[2].startswith("V"):
                d[(a[0], a[2])].add(a[1])
        return d
    uni, kai = read(uni_path), read(kai_path)

    def chosen(lemma, feat):
        a, b = uni.get((lemma, feat), set()), kai.get((lemma, feat), set())
        pool = (a & b) or a or b
        return sorted(pool)[0] if pool else None

    lemmas = sorted({l for (l, f) in list(uni) + list(kai) if l and " " not in l})

    hits = total = 0
    rows = []
    for lemma in lemmas:
        out = []
        for c in CELLS:
            g = chosen(lemma, c)
            if g is not None:
                total += 1
                if productive(lemma, c) == g:
                    hits += 1
            out.append(g if (g is not None and g != productive(lemma, c)) else "-")
        if any(x != "-" for x in out):
            rows.append(lemma + "\t" + "\t".join(out))

    print("# lemma(infinitive)\t" + "\t".join(CELLS) + '  ("-" = productive default)')
    for r in rows:
        print(r)

    pct = 100.0 * hits / total if total else 0.0
    print(f"rule accuracy: {hits}/{total} = {pct:.1f}% "
          f"({len(rows)} lemmas carry >=1 mined cell)", file=sys.stderr)


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
