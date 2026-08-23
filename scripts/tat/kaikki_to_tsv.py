#!/usr/bin/env python3
"""kaikki.org Tatar verbs -> shared TSV, the synthetic (single-word) core:
present (-а/-ә), past definite (-ды/-де), future indefinite / aorist
(-ар/-әр ~ -ыр/-ер), the conditional (-са/-сә) and the verbal-noun
citation (noun-from-verb). Person/number are the six agreement cells.

Kazan Tatar is Cyrillic; the citation is the verbal noun in -у / -ү. Some
kaikki entries are headed by the -рга/-ргә infinitive (or an aorist) rather
than the verbal noun, so every entry is re-keyed to its own noun-from-verb
form — giving one uniform verbal-noun citation for the engine to cite from.

Each kaikki entry lays the positive and the negative column side by side
under IDENTICAL tag-sets (the negative rows carry no 'negative' tag), and
the positive column comes first — so we keep only the FIRST form seen for
each feature, which is the positive paradigm. The perfect (-ган), the
definite future (-ачак), the optative/imperative (kaikki tags these
'error-unrecognized-form') and the negative are left to a later pass."""
import json
import sys

PERSON = {"first-person": "1", "second-person": "2", "third-person": "3"}
NUMBER = {"singular": "SG", "plural": "PL"}
DROP = {"error-unrecognized-form", "table-tags", "inflection-template",
        "romanization", "alternative", "canonical"}


def feature(tags):
    t = {x.lower() for x in tags}
    if t & DROP:
        return None
    if "noun-from-verb" in t:            # verbal-noun citation
        return "V;NFIN"
    p = next((v for k, v in PERSON.items() if k in t), None)
    n = next((v for k, v in NUMBER.items() if k in t), None)
    if not (p and n):
        return None
    if "conditional" in t:
        return f"V;COND;{p};{n}"
    if "indicative" not in t:
        return None
    if "present" in t:
        return f"V;PRS;{p};{n}"
    if "past" in t and "definite" in t:          # -ды/-де definite past
        return f"V;PST;{p};{n}"
    if "future" in t and "indefinite" in t:      # -ар/-әр ~ -ыр/-ер aorist
        return f"V;FUT;{p};{n}"
    return None


def main(path):
    for line in open(path):
        try:
            e = json.loads(line)
        except ValueError:
            continue
        if e.get("pos") != "verb":
            continue
        word = e.get("word", "").strip()
        if not word or " " in word:
            continue
        # First form seen per feature wins (positive column precedes negative).
        chosen = {}
        for f in e.get("forms", []):
            form = f.get("form", "").strip()
            if not form or " " in form or form == "-":
                continue
            ft = feature(f.get("tags", []))
            if ft and ft not in chosen:
                chosen[ft] = form
        # Re-key the whole paradigm to the verbal-noun citation.
        lemma = chosen.get("V;NFIN", word)
        if " " in lemma or not (lemma.endswith("у") or lemma.endswith("ү")):
            continue
        for ft, form in sorted(chosen.items()):
            print(f"{lemma}\t{form}\t{ft}")


if __name__ == "__main__":
    main(sys.argv[1])
