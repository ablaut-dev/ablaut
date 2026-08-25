#!/usr/bin/env python3
"""Turn the kaikki.org (Wiktextract) Hawaiian verb dump into the golden
oracle TSV for the Hawaiian *derivational* engine.

Hawaiian marks tense/aspect/mood with free preverbal particles (ua V, ke V
nei, e V ana, …) while the verb stem stays invariant — that is periphrasis,
out of scope here exactly as analytic TAM is treated elsewhere in the repo.
The only *bound* verbal morphology is derivational, and Wiktionary records it
as human-curated "Derived terms" and passive-tagged `forms` hanging off each
verb lemma. This script mines three productive, lemma-linked paradigms:

  * V;CAUS  — the causative/simulative prefix hoʻo- (allomorphs hō-/hoʻā-…),
              taken from every single-word derived term of shape hoʻo…/hō…;
  * V;RDP   — full reduplication (base+base), the productive
              plural/intensive/frequentative stem;
  * V;PASS  — the -ʻia passive and its lexical -a/-na allomorphs, from
              `forms` tagged "passive".

Emit `lemma <TAB> form <TAB> features`, the format the shared golden harness
consumes. Each row is an independent Wiktionary attestation, never an engine
output — the engine is scored against these, not the reverse.
"""
import json
import sys

CAUS_PREFIXES = ("hoʻo", "hoʻā", "hō", "hoʻ")


def is_word(w: str) -> bool:
    return bool(w) and " " not in w


def main() -> None:
    path = sys.argv[1]
    rows = []
    with open(path, encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            d = json.loads(line)
            if d.get("pos") != "verb":
                continue
            base = d.get("word", "")
            if not is_word(base):
                continue

            # V;CAUS: single-word hoʻo-/hō- derived terms.
            seen = set()
            for der in d.get("derived", []):
                w = der.get("word", "")
                if w in seen or w == base or not is_word(w):
                    continue
                if any(w.startswith(p) for p in CAUS_PREFIXES):
                    rows.append((base, w, "V;CAUS"))
                    seen.add(w)
                # V;RDP: full reduplication base+base.
                if w == base + base:
                    rows.append((base, w, "V;RDP"))

            # V;PASS: forms tagged "passive" (the lexical -a/-na/-ʻia residue).
            for fo in d.get("forms", []):
                if "passive" in fo.get("tags", []):
                    pf = fo.get("form", "")
                    if is_word(pf) and pf != base:
                        rows.append((base, pf, "V;PASS"))

    seen = set()
    for lemma, form, feat in sorted(set(rows)):
        key = (lemma, form, feat)
        if key in seen:
            continue
        seen.add(key)
        print(f"{lemma}\t{form}\t{feat}")


if __name__ == "__main__":
    main()
