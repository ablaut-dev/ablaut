#!/usr/bin/env python3
"""Derive Scottish Gaelic gold (kaikki.tsv) and mined principal parts
(verbs.tsv) from the kaikki.org Scottish Gaelic verb dump.

kaikki prints forms with their initial mutations and particles baked
in (ghlan, chuir, dh'òl, ag ràdh). The past, conditional and relative
future are lenited in the independent column; we de-lenite them to the
unmutated citation form (the engine's `lenite` re-applies the display
mutation), exactly as the Irish pipeline does. Future, imperative and
non-finite forms are already citation forms and are left untouched.
"""
import json
import sys
from collections import defaultdict

SRC = sys.argv[1] if len(sys.argv) > 1 else "data/gla/kaikki.jsonl"
GOLD = "data/gla/kaikki.tsv"
VERBS = "data/gla/verbs.tsv"

LENITABLE = set("bcdfgmpst")

# The copula / substantive verb (*bi*, *is*) and its impersonal/negative
# paradigm rows: wholly periphrastic and out of scope (the present is
# `tha mi …`, the conditional `bhithinn`). kaikki lists them as verbs;
# they leak only a couple of *bi*-suppletive conditional cells.
EXCLUDE = {"bi", "is", "thathar", "nach", "bhathar", "robh", "rabhar"}


def delenite(f, lemma):
    """Strip a leading dh'/d' particle and undo initial lenition.

    Lenition inserts an `h` after the stem's initial consonant; we only
    strip it when the lemma's own initial is unlenited (glan → ghlan →
    glan), never when the root genuinely begins consonant+h (bhòt,
    thig) and the surface shares that same initial consonant."""
    for pre in ("dh'", "d'", "dh’", "d’"):
        if f.startswith(pre):
            f = f[len(pre):]
            break
    if len(f) >= 2 and f[0] in LENITABLE and f[1] == "h":
        root_has_h = len(lemma) >= 2 and lemma[0] == f[0] and lemma[1] == "h"
        if not root_has_h:
            f = f[0] + f[2:]
    return f


def bad(form):
    return (
        not form
        or form == "-"
        or " " in form
        or "{" in form
        or "}" in form
        or "\t" in form
    )


def classify(tags, form):
    """Map a kaikki tag-set to (feature, delenite?) or None to skip.

    Returns a list of (feature, delenite) because a couple of raw tag
    bundles are disambiguated by the form's own suffix (the imperative
    1pl/2pl/3 collapse onto identical tags in kaikki)."""
    T = set(tags)
    if T & {"inflection-template", "table-tags", "mutation", "mutation-radical"}:
        return []
    # Copula / substantive-verb periphrasis and non-synthetic moods.
    if T & {"present", "negative", "interrogative", "affirmative", "emphatic",
            "alternative"}:
        return []
    # Spurious imperative rows duplicating the non-finite forms.
    if "imperative" in T and ("participle" in T or "noun-from-verb" in T):
        return []

    # Non-finite principal parts.
    if T == {"noun-from-verb"}:
        return [("V;VN", False)]
    if T == {"participle", "past"}:
        return [("V.PTCP", False)]

    # Bare principal parts (verbs without a full table).
    if T == {"past"}:
        return [("V;PST;IND", True)]
    if T == {"future"}:
        return [("V;FUT;IND", False)]

    # Past.
    if T == {"independent", "indicative", "past", "personal"}:
        return [("V;PST;IND", True)]
    if T == {"impersonal", "independent", "indicative", "past"}:
        return [("V;PST;IMPRS", True)]
    if T == {"dependent", "past", "personal"}:
        return [("V;PST;DEP", True)]
    if T == {"dependent", "impersonal", "past"}:
        return [("V;PST;IMPRS", True)]

    # Future.
    if T == {"future", "independent", "indicative", "personal"}:
        return [("V;FUT;IND", False)]
    if T == {"future", "impersonal", "independent", "indicative"}:
        return [("V;FUT;IMPRS", False)]
    if T == {"dependent", "future", "personal"}:
        return [("V;FUT;DEP", False)]
    if T == {"dependent", "future", "impersonal"}:
        return [("V;FUT;IMPRS", False)]
    # Relative future (kaikki mis-tags it error-unrecognized-form).
    if T == {"error-unrecognized-form", "independent", "indicative", "personal"}:
        return [("V;FUT;REL", True)]

    # Conditional (independent).
    if T == {"conditional", "first-person", "independent", "singular"}:
        return [("V;COND;1SG", True)]
    if T == {"conditional", "first-person", "independent", "plural"}:
        return [("V;COND;1PL", True)]
    if T == {"conditional", "impersonal", "independent"}:
        return [("V;COND;IMPRS", True)]
    if T == {"conditional", "error-unrecognized-form", "independent", "personal"}:
        return [("V;COND;3", True)]
    # Conditional (dependent) — not lenited.
    if T == {"dependent", "first-person", "singular"}:
        return [("V;COND;1SG", False)]
    if T == {"dependent", "first-person", "plural"}:
        return [("V;COND;1PL", False)]
    if T == {"dependent", "impersonal"}:
        return [("V;COND;IMPRS", False)]
    if T == {"dependent", "error-unrecognized-form", "personal"}:
        return [("V;COND;3", False)]

    # Imperative.
    if T == {"first-person", "imperative", "independent", "singular"}:
        return [("V;IMP;1SG", False)]
    if T == {"imperative", "independent", "second-person", "singular"}:
        return [("V;IMP;2SG", False)]
    if T == {"imperative", "independent", "plural", "third-person"}:
        return [("V;IMP;3", False)]
    if T == {"dependent", "imperative", "impersonal", "independent"}:
        return [("V;IMP;IMPRS", False)]
    if T == {"imperative", "impersonal", "independent"}:
        # 1pl (-amaid), 2pl (-aibh/-ibh) and 3 (-adh/-eadh) share tags.
        if form.endswith("amaid"):
            return [("V;IMP;1PL", False)]
        if form.endswith("aibh") or form.endswith("ibh"):
            return [("V;IMP;2PL", False)]
        return [("V;IMP;3", False)]

    return []


def main():
    lines = [json.loads(l) for l in open(SRC)]
    # gold[lemma][feature] = set(forms)
    gold = defaultdict(lambda: defaultdict(set))
    # principal parts
    parts = {}
    for entry in lines:
        lemma = entry["word"].strip()
        if bad(lemma) or "-" == lemma or lemma in EXCLUDE:
            continue
        pp = {}
        for f in entry.get("forms", []):
            form = f.get("form", "")
            tags = f.get("tags", [])
            for feature, dl in classify(tags, form):
                surface = delenite(form, lemma) if dl else form
                if bad(surface):
                    continue
                gold[lemma][feature].add(surface)
                # collect principal parts
                if feature == "V;PST;IND":
                    pp.setdefault("past", surface)
                elif feature == "V;FUT;IND":
                    pp.setdefault("future", surface)
                elif feature == "V;VN":
                    pp.setdefault("vn", surface)
                elif feature == "V.PTCP":
                    pp.setdefault("ptcp", surface)
        if pp:
            parts[lemma] = pp

    # Write gold.
    rows = 0
    with open(GOLD, "w") as fh:
        for lemma in sorted(gold):
            for feature in sorted(gold[lemma]):
                for form in sorted(gold[lemma][feature]):
                    fh.write(f"{lemma}\t{form}\t{feature}\n")
                    rows += 1

    # Write mined principal parts.
    with open(VERBS, "w") as fh:
        fh.write("# lemma\tpast\tfuture\tvn\tptcp  (mined principal parts, "
                 "delenited past; \"-\" = derive)\n")
        for lemma in sorted(parts):
            pp = parts[lemma]
            fh.write("\t".join([
                lemma,
                pp.get("past", "-"),
                pp.get("future", "-"),
                pp.get("vn", "-"),
                pp.get("ptcp", "-"),
            ]) + "\n")

    print(f"lemmas in gold: {len(gold)}", file=sys.stderr)
    print(f"gold rows: {rows}", file=sys.stderr)
    print(f"principal-part rows: {len(parts)}", file=sys.stderr)


if __name__ == "__main__":
    main()
