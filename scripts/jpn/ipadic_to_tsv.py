#!/usr/bin/env python3
"""Convert the mecab-ipadic Verb.csv (EUC-JP) into the second-oracle TSV,
in the same lemma<TAB>form<TAB>features schema as the kaikki side.

IPADIC rows are surface tokens tagged with 活用型 (conjugation type, e.g.
五段・カ行イ音便) and 活用形 (form, e.g. 未然形 / 連用形 / 基本形 / 仮定形 /
命令ｅ). We emit the three slots that align with kaikki's top-level forms:

    V;NFIN  基本形                     (= the lemma)
    V;CONT  連用形
    V;PST   plain past た/だ-form, reconstructed from 連用タ接続 (the onbin
            stem, e.g. 書い / 泳い / 死ん / 買っ) plus the voiced auxiliary
            だ for the ガ/ナ/バ/マ rows and た elsewhere; 一段/サ変/カ変 take
            連用形 + た.

The full katsuyou-kei (未然形/仮定形/命令形) IPADIC also carries are *not*
emitted: kaikki attests them only in its classical table, so they never
survive the two-oracle agreement and would be dead rows.

Usage: iconv -f EUC-JP -t UTF-8 Verb.csv \
         | python3 scripts/jpn/ipadic_to_tsv.py > data/jpn/ipadic.tsv
   (or pass the already-UTF-8 file as argv[1])
"""
import collections
import sys

VOICED_ROWS = ("ガ行", "ナ行", "バ行", "マ行")


def load(fh):
    """base -> {活用形: set(surface)} and base -> 活用型."""
    forms = collections.defaultdict(lambda: collections.defaultdict(set))
    ctype = {}
    for line in fh:
        p = line.rstrip("\n").split(",")
        if len(p) < 11 or p[4] != "動詞":
            continue
        surface, katsuyoukei, base = p[0], p[9], p[10]
        forms[base][katsuyoukei].add(surface)
        ctype.setdefault(base, p[8])
    return forms, ctype


def past_forms(base, forms, ctype):
    t = ctype[base]
    if "一段" in t or "サ変" in t or "カ変" in t:
        return {r + "た" for r in forms[base].get("連用形", set())}
    onbin = forms[base].get("連用タ接続")
    if not onbin:
        return {r + "た" for r in forms[base].get("連用形", set())}
    aux = "だ" if any(r in t for r in VOICED_ROWS) else "た"
    return {o + aux for o in onbin}


def main(argv):
    fh = open(argv[1], encoding="utf-8") if len(argv) > 1 else sys.stdin
    forms, ctype = load(fh)
    seen = set()
    out = sys.stdout
    for base in forms:
        rows = [(base, "V;NFIN")]
        for r in sorted(forms[base].get("連用形", set())):
            rows.append((r, "V;CONT"))
        for r in sorted(past_forms(base, forms, ctype)):
            rows.append((r, "V;PST"))
        for form, feat in rows:
            key = (base, form, feat)
            if key not in seen and form:
                seen.add(key)
                out.write(f"{base}\t{form}\t{feat}\n")


if __name__ == "__main__":
    main(sys.argv)
