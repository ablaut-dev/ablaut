#!/usr/bin/env python3
"""Convert the kaikki.org Gujarati verb JSONL into the shared
`lemma ⇥ form ⇥ features` TSV, using the same UniMorph feature bundles
scripts/guj/unimorph_to_tsv.py emits, so the harness can score against
it.

kaikki guj is a spot check, not an agreement partner: it is the same
English-Wiktionary lineage as UniMorph guj (both from the `gu-conj`
template), and Wiktextract could not map the Gujarati column headers, so
most inflected cells are tagged `error-unrecognized-form` and lose their
person/number. This converter is therefore conservative — it emits only
the categories whose tags are unambiguous, and skips the ones kaikki
pollutes (e.g. the 2nd-person future, which shares its tag set with the
polite imperative કરજે):

* the **past** (કર્યું) and **past progressive** (કરતું હતું);
* the **first- and third-person future** (કરીશ, કરીશું, કરશે);
* the **present progressive** with a person tag (કરું છું);
* the two **converbs** (LGSPEC1 કરી, LGSPEC2 કરીને), the **verbal noun**
  (V.MSDR કરવાનું) and the **conditional** (LGSPEC3 કરત).

Usage: python3 scripts/guj/kaikki_to_tsv.py data/guj/kaikki-verbs.jsonl
"""

import json
import re
import sys

GUJ = re.compile(r"[઀-૿]")
SKIP = {"romanization", "table-tags", "inflection-template"}


def is_guj(s):
    return bool(GUJ.search(s))


def normalize(form):
    return form.replace("ઁ", "ં")


def number(tags):
    return "PL" if "plural" in tags else "SG"


def person(tags):
    for t, p in (("first-person", "1"), ("second-person", "2"), ("third-person", "3")):
        if t in tags:
            return p
    return None


def bundles(form, tags):
    """Zero or more UniMorph bundles for one kaikki (form, tags) row, or
    an empty list to drop it."""
    t = set(tags)
    if t & SKIP or "negative" in t:
        return []
    if not is_guj(form):
        return []

    if "noun-from-verb" in t:
        return ["V;V.MSDR"]
    if "conjunctive" in t:
        return ["V;LGSPEC1"]
    if "consecutive" in t:
        return ["V;LGSPEC2"]

    if "past" in t:
        return ["V;IND;PST;PROG;POS"] if "progressive" in t else ["V;IND;PST;POS"]

    # The single-word conditional (કરત); the multiword કરતું હોત is
    # LGSPEC4, which we do not spot-check.
    if "counterfactual" in t and " " not in form:
        return ["V;LGSPEC3"]

    p = person(t)
    n = number(t)

    if "future" in t and p in ("1", "3"):
        # 2nd-person future shares its tags with the polite imperative
        # (કરજે), so it is skipped.
        return [f"V;IND;FUT;POS;{p};{'SG+PL' if p == '3' else n}"]

    if "present" in t and "progressive" in t and p is not None:
        return [f"V;IND;PRS;PROG;POS;{p};{'SG+PL' if p == '3' else n}"]

    return []


def main(path):
    rows = set()
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            d = json.loads(line)
            lemma = d.get("word")
            if not lemma or not is_guj(lemma):
                continue
            for f in d.get("forms", []):
                form = (f.get("form") or "").strip()
                tags = f.get("tags")
                if not form or not tags:
                    continue
                for features in bundles(form, tags):
                    rows.add((normalize(lemma), normalize(form), features))
    out = sys.stdout
    for lemma, form, features in sorted(rows):
        out.write(f"{lemma}\t{form}\t{features}\n")


if __name__ == "__main__":
    main(sys.argv[1])
