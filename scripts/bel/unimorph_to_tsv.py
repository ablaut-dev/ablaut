#!/usr/bin/env python3
"""Normalise the raw UniMorph Belarusian table into the shared TSV.

Keep verb rows (feature starts with V), strip the combining-acute stress
marks UniMorph writes but Belarusian orthography does not, normalise the
apostrophe glyphs to U+0027, and drop multi-word (analytic) forms — the
imperfective future бу́ду … + infinitive — so only the synthetic one-word
paradigm survives, matching the kaikki adapter.
"""
import sys
import unicodedata


def norm(s):
    # Strip only the acute/grave stress marks, NOT every combining mark:
    # Belarusian ў decomposes to у + combining breve (U+0306), so a blanket
    # Mn strip would destroy it. Re-compose (NFC) afterwards.
    s = unicodedata.normalize("NFD", s)
    s = "".join(c for c in s if c not in ("́", "̀"))
    s = unicodedata.normalize("NFC", s)
    return s.replace("’", "'").replace("ʼ", "'").replace("`", "'")


def main(path):
    for line in open(path):
        a = line.rstrip("\n").split("\t")
        if len(a) < 3:
            continue
        lemma, form, feat = norm(a[0]), norm(a[1]), a[2].strip()
        if not feat.startswith("V"):
            continue
        if not lemma or not form or " " in lemma or " " in form:
            continue
        print(f"{lemma}\t{form}\t{feat}")


if __name__ == "__main__":
    main(sys.argv[1])
