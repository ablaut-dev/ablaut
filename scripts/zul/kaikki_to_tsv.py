#!/usr/bin/env python3
"""Convert the kaikki.org (Wiktextract) Zulu verb extraction to the shared
`lemma ⇥ form ⇥ features` TSV, in the canonical schema shared with the
UniMorph adapter.

kaikki keys each entry on the bare stem (`fika`, `fa`) — the same lemma
UniMorph uses — and lists a large `forms` table. Only the cleanly-tagged
slots are emitted; the bulk of the present/future forms carry the
`error-unrecognized-form` tag and no subject, so they cannot be aligned
per-cell and are dropped. The reliably-tagged, per-person slots kaikki
does expose, mapped onto the UniMorph canonical bundles:

  infinitive                              → V;NFIN
  imperative;singular / plural            → V;IMP;2SG / V;IMP;2PL
  <person>;present;subjunctive            → V;<SUBJ>;SBJV   (final -e)
  <person>;past;subjunctive               → V;<SUBJ>;RMT_PST

kaikki labels the remote past ("ngafika") a *past subjunctive*; after
macron stripping it is exactly UniMorph's `RMT;PST` (`ngāfika`). Object-
concord forms (`-fike`), the negatives (UniMorph has none, so they never
double-cover) and every `error-unrecognized-form` are skipped.

Usage: python3 scripts/zul/kaikki_to_tsv.py data/zul/kaikki-verbs.jsonl
"""

import json
import sys

MACRON = str.maketrans({"ā": "a", "ē": "e", "ī": "i", "ō": "o", "ū": "u"})


def person(tags):
    t = set(tags)
    p = "1" if "first-person" in t else "2" if "second-person" in t else None
    n = "SG" if "singular" in t else "PL" if "plural" in t else None
    return f"{p}{n}" if p and n else None


def canonical(tags):
    """Map a kaikki tag set to a canonical bundle, or None to skip."""
    t = set(tags)
    if "error-unrecognized-form" in t:
        return None
    if "negative" in t or "object-concord" in t:
        return None
    if t & {"canonical", "table-tags", "inflection-template", "alternative"}:
        return None
    if "infinitive" in t:
        return "V;NFIN"
    if "imperative" in t:
        if "singular" in t:
            return "V;IMP;2SG"
        if "plural" in t:
            return "V;IMP;2PL"
        return None
    if "subjunctive" in t:
        subj = person(tags)
        if subj is None:
            return None
        if "past" in t:
            return f"V;{subj};RMT_PST"
        if "present" in t:
            return f"V;{subj};SBJV"
    return None


def main(path):
    rows = set()
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            d = json.loads(line)
            lemma = d.get("word")
            if not lemma:
                continue
            for fm in d.get("forms", []):
                form = fm.get("form", "")
                if not form or " " in form or form.startswith("-"):
                    continue
                feat = canonical(fm.get("tags", []))
                if not feat:
                    continue
                rows.add((lemma, form.translate(MACRON), feat))
    out = sys.stdout
    for lemma, form, feat in sorted(rows):
        out.write(f"{lemma}\t{form}\t{feat}\n")


if __name__ == "__main__":
    main(sys.argv[1])
