#!/usr/bin/env python3
"""Mine the Persian present-stem list (data/pes/verbs.tsv) from the
kaikki.org conjugation tables.

Persian's past stem is regular (infinitive minus its final ن) and the
`ـیدن` class has a productive present stem (infinitive minus ـیدن), so
those need no entry. Everything else — the ~200 verbs whose present stem
is not derivable — is what this file emits: one row per simple verb,

    infinitive <TAB> present_stem <TAB> imperative_override

The *bound* present stem (the shape endings attach to: گو‌ی for گفتن, نه
ده for دادن) is read off the standard aorist 1sg (اورش longest variant,
which is the literary one, minus its final م). The imperative override is
filled only when kaikki's standard imperative 2sg is not the regular
بـ + stem (گفتن → بگو, آمدن → بیا).

Usage: python3 scripts/pes/mine_verbs.py data/pes/kaikki-verbs.jsonl > data/pes/verbs.tsv
"""
import json
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
from norm import normalize  # noqa: E402


def be_prefix(stem):
    """Mirror pes.rs be_prefix: the regular بـ imperative/subjunctive."""
    if stem.startswith("آ"):
        return "بیا" + stem[1:]
    if stem.startswith("ا"):
        return "بی" + stem[1:]
    return "ب" + stem


def main():
    path = sys.argv[1]
    rows = {}
    for line in open(path, encoding="utf-8"):
        d = json.loads(line)
        forms = d.get("forms", [])
        w = normalize(d["word"])
        if " " in w:
            continue  # compounds reuse the simple verb's stem
        inf = None
        aor1 = set()
        imp = set()
        for f in forms:
            t = set(f.get("tags", []))
            if f.get("tags") == ["infinitive"] and inf is None:
                inf = normalize(f["form"])
            if {"aorist", "first-person", "singular"} <= t:
                aor1.add(normalize(f["form"]))
            if {"imperative", "second-person", "singular"} <= t:
                imp.add(normalize(f["form"]))
        if not inf or not aor1 or not inf.endswith("ن"):
            continue
        if w in rows:
            continue
        # bound present stem = longest (literary) aorist 1sg minus final م
        a = max(aor1, key=len)
        bstem = a[:-1] if a.endswith("م") else a
        if not bstem or " " in bstem:
            # A spaced stem means a preverb compound written solid in the
            # lemma (برداشتن ~ بر می‌دارم); out of scope for the simple
            # present-stem table.
            continue
        # skip the productive یدن class whose rule already yields bstem
        if inf.endswith("یدن") and bstem == inf[:-3]:
            continue
        # imperative override only when the regular rule misses
        override = "-"
        regular = be_prefix(bstem)
        if imp and regular not in imp:
            override = min(imp, key=len)  # shortest = the plain imperative
        rows[w] = (bstem, override)

    print("# Persian present-stem list — mined by scripts/pes/mine_verbs.py")
    print("# infinitive\tpresent_stem\timperative_override")
    for inf in sorted(rows):
        bstem, override = rows[inf]
        print(f"{inf}\t{bstem}\t{override}")


if __name__ == "__main__":
    main()
