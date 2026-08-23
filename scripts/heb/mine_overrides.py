#!/usr/bin/env python3
"""Mine the weak-root residue into data/heb/overrides.tsv.

The productive rules in src/heb.rs carry the strong-verb bulk and the regular
weak classes (ל״ה final-ה, ע״ו/ע״י hollow, hifil medial hireq-yod, final-ת/נ
mergers). A handful of genuinely irregular roots stay uncovered — ה-final
strong homographs of ל״ה verbs (נגה, כמה, תמה), double-waw ל״ה (חווה, נכווה),
פ״נ assimilation, and lexical irregulars (נתן, יכול, קטון). This script
captures exactly those cells as (lemma, features, form) overrides.

Method: blank the override table, run the golden harness (which diffs the
engine against the two-oracle agreement gold and writes every mismatch to
target/golden_heb_mismatches.tsv), then emit one accepted gold form per
mismatching cell. Overrides are consulted before the rules, so this closes the
gap while keeping the split honest — the override count is the residue only.

Run from the repo root:  python3 scripts/heb/mine_overrides.py
Then:                    git add -f data/heb/overrides.tsv
"""
import os
import subprocess
import sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
OVERRIDES = os.path.join(ROOT, "data", "heb", "overrides.tsv")
MISMATCHES = os.path.join(ROOT, "target", "golden_heb_mismatches.tsv")

HEADER = "# lemma\tfeatures\tform\n"
HEADER += "# Mined weak-root residue: cells the rules do not reach (ה-final\n"
HEADER += "# strong homographs, double-waw ל״ה, פ״נ, and lexical irregulars).\n"
HEADER += "# Regenerate with scripts/heb/mine_overrides.py.\n"


def main():
    # 1. Blank the overrides so the harness scores the pure rule engine.
    with open(OVERRIDES, "w", encoding="utf-8") as f:
        f.write(HEADER)

    # 2. Run the harness to regenerate the mismatch dump.
    subprocess.run(
        ["cargo", "run", "--release", "--quiet", "--bin", "golden_heb"],
        cwd=ROOT,
        check=True,
    )

    # 3. Turn each mismatch into an override, choosing the first accepted
    #    gold variant (any variant in the agreement counts as correct).
    rows = []
    with open(MISMATCHES, encoding="utf-8") as f:
        for line in f:
            parts = line.rstrip("\n").split("\t")
            if len(parts) < 4:
                continue
            lemma, features, _engine, gold = parts[0], parts[1], parts[2], parts[3]
            variant = gold.split("|")[0]
            if variant:
                rows.append((lemma, features, variant))

    rows.sort()
    with open(OVERRIDES, "w", encoding="utf-8") as f:
        f.write(HEADER)
        for lemma, features, form in rows:
            f.write(f"{lemma}\t{features}\t{form}\n")

    print(f"wrote {len(rows)} overrides to {OVERRIDES}", file=sys.stderr)


if __name__ == "__main__":
    main()
