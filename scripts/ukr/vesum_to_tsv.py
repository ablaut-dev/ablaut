#!/usr/bin/env python3
"""Convert the VESUM (brown-uk/dict_uk) expanded full-form dictionary
dict_corp_vis.txt into the lemma<TAB>form<TAB>features TSV that the
Ukrainian golden harness reads. This is the independent, non-Wiktionary
second oracle.

Usage: python3 scripts/ukr/vesum_to_tsv.py data/ukr/dict_corp_vis.txt \
           > data/ukr/vesum.tsv

Format of dict_corp_vis.txt: a lemma line at column 0 (its own inflected
form, usually the infinitive) followed by 2-space-indented form lines, each
"form  verb:...:tag" with an optional trailing "# semantic-tags" comment.

Shared schema with scripts/ukr/kaikki_to_tsv.py:
  V;NFIN, V;IND;PRS;{SG,PL};{1,2,3},
  V;IND;PST;{MASC,FEM,NEUT};SG, V;IND;PST;PL, V;IMP;{SG,PL};{1,2,3}.

The non-past slot V;IND;PRS unifies the imperfective present (pres) and
the perfective non-past, which VESUM tags "futr"; the imperfective
synthetic future in -тиму (also "futr") is not this slot and is skipped.
Reflexive verbs carry a "rev" tag, dropped before aspect. Forms flagged
:bad are skipped; other stylistic variants (:short, :subst, :coll) are
kept, since the harness unions variants and scores only agreed slots.
"""
import sys

NUM = {"s": "SG", "p": "PL"}
GEN = {"m": "MASC", "f": "FEM", "n": "NEUT"}


def features(tag):
    """Map a VESUM verb tag to a feature string, or None."""
    parts = tag.split(":")
    if not parts or parts[0] != "verb" or "bad" in parts:
        return None
    parts = parts[1:]
    if parts and parts[0] == "rev":
        parts = parts[1:]
    if not parts or parts[0] not in ("imperf", "perf"):
        return None
    aspect = parts[0]
    rest = parts[1:]
    if not rest:
        return None
    kind = rest[0]
    if kind == "inf":
        return "V;NFIN"
    if kind == "impr" and len(rest) >= 3:
        n, p = NUM.get(rest[1]), rest[2]
        return f"V;IMP;{n};{p}" if n and p in "123" else None
    if kind == "pres" and len(rest) >= 3:
        n, p = NUM.get(rest[1]), rest[2]
        return f"V;IND;PRS;{n};{p}" if n and p in "123" else None
    if kind == "futr" and aspect == "perf" and len(rest) >= 3:
        n, p = NUM.get(rest[1]), rest[2]
        return f"V;IND;PRS;{n};{p}" if n and p in "123" else None
    if kind == "past" and len(rest) >= 2:
        if rest[1] == "p":
            return "V;IND;PST;PL"
        g = GEN.get(rest[1])
        return f"V;IND;PST;{g};SG" if g else None
    return None


def main(path):
    seen = set()
    lemma = None
    for line in open(path):
        raw = line.rstrip("\n")
        if not raw.strip():
            continue
        indented = raw[0] in (" ", "\t")
        toks = raw.split()
        if len(toks) < 2:
            continue
        form, tag = toks[0], toks[1]
        if not indented:
            lemma = form
        if lemma is None or " " in form:
            continue
        feat = features(tag)
        if feat is None:
            continue
        row = (lemma, form, feat)
        if row not in seen:
            seen.add(row)
            sys.stdout.write(f"{lemma}\t{form}\t{feat}\n")


if __name__ == "__main__":
    main(sys.argv[1])
