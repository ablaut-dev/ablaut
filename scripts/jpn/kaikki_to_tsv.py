#!/usr/bin/env python3
"""Convert the kaikki.org (Wiktextract) Japanese verb JSONL into the
lemma<TAB>form<TAB>features TSV the Japanese golden harness reads.

kaikki expands, per verb head, a handful of *top-level* forms tagged by
en.wiktionary's ``ja-verb`` template: the citation/terminal form (the
word itself), the ``stem`` (連用形 / masu-stem, e.g. 書く → 書き) and the
plain ``past`` た-form (書く → 書いた). Those three are the surface strings
a non-Wiktionary morphological lexicon (IPADIC) also attests, so they are
the alignable slot schema:

    V;NFIN  終止形  dictionary form   (= the lemma)
    V;CONT  連用形  continuative      (kaikki ``stem``)
    V;PST   —      plain past た-form (kaikki ``past``)

The rich per-verb conjugation *tables* kaikki carries are either the
classical (bungo) katsuyou table — wrong orthography for the modern
lexicon (会は vs 会わ) — or the full modern table, present for only ~120
verbs; neither is a broad, modern, IPADIC-alignable source, so we take
only the three top-level slots. Romanised and kana-gloss duplicates are
dropped; we keep the kanji citation spelling.

Usage: python3 scripts/jpn/kaikki_to_tsv.py data/jpn/kaikki-verbs.jsonl \
           > data/jpn/kaikki.tsv
"""
import json
import sys


def is_ja(s):
    """A real Japanese surface string (not a romaji gloss)."""
    return bool(s) and not all(ord(c) < 128 for c in s)


def main(path):
    seen = set()
    out = sys.stdout
    for line in open(path, encoding="utf-8"):
        entry = json.loads(line)
        lemma = entry.get("word", "").strip()
        if not is_ja(lemma) or " " in lemma:
            continue
        # Top-level (non-conjugation-table) forms only.
        top = [f for f in entry.get("forms", []) if not f.get("source")]

        def pick(tag):
            for f in top:
                tags = f.get("tags") or []
                if tag in tags and "romanization" not in tags:
                    form = f.get("form", "").strip()
                    if is_ja(form) and " " not in form:
                        return form
            return None

        stem = pick("stem")
        past = pick("past")
        rows = [(lemma, "V;NFIN")]
        if stem:
            rows.append((stem, "V;CONT"))
        if past:
            rows.append((past, "V;PST"))
        for form, feat in rows:
            row = (lemma, form, feat)
            if row not in seen:
                seen.add(row)
                out.write(f"{lemma}\t{form}\t{feat}\n")


if __name__ == "__main__":
    main(sys.argv[1])
