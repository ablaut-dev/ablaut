#!/usr/bin/env python3
"""Mine the Hawaiian override + principal-parts layers from the oracle.

The engine derives the causative, reduplicated and passive stems by rule
(see src/haw.rs). A handful of derived terms are lexicalised — causatives
fused with reduplication (hele→hoʻohelehele), stems whose vowel lengthens
under prefixation (mahele→hoʻomāhele), ʻokina/long-vowel edge cases — that
no productive rule should be forced to predict. This script replays the
Rust rules in Python, and writes:

  * data/haw/overrides.tsv  — lemma ⇥ feature ⇥ form, one row per oracle
    form the rules miss (consulted before the rules in the engine);
  * data/haw/parts.tsv      — the distinct verb-lemma inventory (col 1),
    the lexicon the reverse index is built from.

Run after scripts/haw/fetch_kaikki.sh. Idempotent.
"""
import sys

VOWELS = set("aeiou")
LONG = set("āēīōū")


def causative_candidates(base: str) -> list:
    """Mirror of haw::causative — the candidate hoʻo- surface forms."""
    if not base:
        return []
    cands = ["hoʻo" + base]
    v0, rest = base[0], base[1:]
    if v0 == "ʻ":
        cands.append("hō" + base)
    elif v0 == "a":
        cands += ["hoʻā" + rest, "hō" + base]
    elif v0 == "o":
        cands.append("hoʻō" + rest)
    elif v0 == "e":
        cands += ["hoʻē" + rest, "hōʻe" + rest]
    elif v0 == "i":
        cands.append("hoʻī" + rest)
    elif v0 == "u":
        cands.append("hoʻū" + rest)
    elif v0 in LONG:
        cands += ["hō" + base, "hoʻ" + base]
    return cands


def reduplication_candidates(base: str) -> list:
    return [base + base]


def passive_candidates(base: str) -> list:
    return [base + s for s in ("ʻia", "a", "na", "hia", "lia", "mia", "ʻana")]


CANDS = {
    "V;CAUS": causative_candidates,
    "V;RDP": reduplication_candidates,
    "V;PASS": passive_candidates,
}


def main() -> None:
    oracle = sys.argv[1]
    lemmas = set()
    overrides = []
    with open(oracle, encoding="utf-8") as f:
        for line in f:
            parts = line.rstrip("\n").split("\t")
            if len(parts) != 3:
                continue
            lemma, form, feat = parts
            lemmas.add(lemma)
            gen = CANDS.get(feat)
            if gen is None:
                continue
            if form not in gen(lemma):
                overrides.append((lemma, feat, form))

    with open("data/haw/overrides.tsv", "w", encoding="utf-8") as out:
        out.write("# Hawaiian derivational overrides: lemma ⇥ feature ⇥ form.\n")
        out.write("# Lexicalised forms the productive rules in src/haw.rs do not\n")
        out.write("# predict — causative+reduplication fusions, prefix-triggered\n")
        out.write("# vowel lengthening, ʻokina/long-vowel edge cases. Mined by\n")
        out.write("# scripts/haw/mine_overrides.py from the kaikki oracle.\n")
        for lemma, feat, form in sorted(set(overrides)):
            out.write(f"{lemma}\t{feat}\t{form}\n")

    with open("data/haw/parts.tsv", "w", encoding="utf-8") as out:
        out.write("# Hawaiian verb-lemma inventory (col 1): the reverse-lookup\n")
        out.write("# lexicon. Mined from the kaikki verb oracle.\n")
        for lemma in sorted(lemmas):
            out.write(f"{lemma}\n")

    print(f"overrides: {len(set(overrides))}  lemmas: {len(lemmas)}")


if __name__ == "__main__":
    main()
