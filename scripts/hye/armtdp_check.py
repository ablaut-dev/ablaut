#!/usr/bin/env python3
"""Corroborate the hand-supplied core rows against the UD treebank.

scripts/hye/manual.tsv carries the everyday irregulars — գալ, տալ,
ուտել, անել, … — that UniMorph hye does not list, so the two-oracle
agreement cannot check them. This script narrows the treebank spot
check (data/hye/armtdp.tsv) to exactly those lemmas and writes the
subset out, so `cargo run --release --bin golden_hye -- <subset>`
scores the engine on attested corpus tokens for the core verbs alone.

Usage: python3 scripts/hye/armtdp_check.py > target/armtdp_core.tsv
"""

import sys


def main():
    core = set()
    with open("scripts/hye/manual.tsv", encoding="utf-8") as fh:
        for line in fh:
            if line.startswith("#") or not line.strip():
                continue
            core.add(line.split("\t")[0])

    kept = 0
    lemmas = set()
    with open("data/hye/armtdp.tsv", encoding="utf-8") as fh:
        for line in fh:
            lemma = line.split("\t")[0]
            if lemma in core:
                sys.stdout.write(line)
                lemmas.add(lemma)
                kept += 1
    print(
        f"{kept} attested slots over {len(lemmas)} of {len(core)} core lemmas",
        file=sys.stderr,
    )


if __name__ == "__main__":
    main()
