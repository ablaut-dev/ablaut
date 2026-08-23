#!/usr/bin/env python3
"""Regenerate data/amh/overrides.tsv — the mined-override residue.

Single-oracle (UniMorph only), so every gold cell the productive rules miss is
patched with one gold form. The script:

  1. blanks the override table,
  2. rebuilds + runs the golden harness to dump pure-rule mismatches,
  3. writes one gold form per residual cell, and
  4. fully seeds the handful of lemmas that lack principal parts (derived ተ-
     stems, the copula ነው) so they count toward lemma coverage.

Run from the repo root:  python3 scripts/amh/mine_overrides.py
"""
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
GOLD = ROOT / "data/amh/unimorph.tsv"
PARTS = ROOT / "data/amh/parts.tsv"
OVERRIDES = ROOT / "data/amh/overrides.tsv"
MISMATCHES = ROOT / "target/golden_amh_mismatches.tsv"
HEADER = "# lemma\tfeature\tform  (mined residue — regenerate with scripts/amh/mine_overrides.py)\n"


def load_gold():
    """(lemma, features) -> first gold variant."""
    gold = {}
    for line in GOLD.read_text(encoding="utf-8").splitlines():
        parts = line.split("\t")
        if len(parts) < 3 or not parts[2].startswith("V"):
            continue
        lemma, form, feats = parts[0].strip(), parts[1].strip(), parts[2].strip()
        gold.setdefault((lemma, feats), form)  # keep first spelling
    return gold


def lemmas_with_parts():
    out = set()
    for line in PARTS.read_text(encoding="utf-8").splitlines():
        if line.startswith("#") or not line.strip():
            continue
        out.add(line.split("\t")[0])
    return out


def run_harness():
    OVERRIDES.write_text(HEADER, encoding="utf-8")  # blank table
    subprocess.run(
        ["cargo", "run", "--release", "--quiet", "--bin", "golden_amh", "--",
         str(GOLD)],
        cwd=ROOT, check=True, stdout=subprocess.DEVNULL,
    )


def main():
    gold = load_gold()
    have_parts = lemmas_with_parts()

    run_harness()

    entries = {}  # (lemma, feats) -> form

    # 1. Residual rule mismatches for lemmas that DO have principal parts.
    for line in MISMATCHES.read_text(encoding="utf-8").splitlines():
        cols = line.split("\t")
        if len(cols) < 4:
            continue
        lemma, feats = cols[0], cols[1]
        form = gold.get((lemma, feats))
        if form:
            entries[(lemma, feats)] = form

    # 2. Fully seed lemmas that lack principal parts.
    seedless = sorted({l for (l, _f) in gold} - have_parts)
    for (lemma, feats), form in gold.items():
        if lemma in seedless:
            entries[(lemma, feats)] = form

    lines = [HEADER]
    for (lemma, feats), form in sorted(entries.items()):
        lines.append(f"{lemma}\t{feats}\t{form}\n")
    OVERRIDES.write_text("".join(lines), encoding="utf-8")

    print(f"wrote {len(entries)} overrides "
          f"({len(seedless)} seedless lemmas fully covered) to {OVERRIDES}",
          file=sys.stderr)


if __name__ == "__main__":
    main()
