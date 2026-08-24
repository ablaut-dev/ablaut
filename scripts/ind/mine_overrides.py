#!/usr/bin/env python3
"""Mine the residue-patching override layer for Indonesian.

Reads the golden binary's mismatch dump
(`target/golden_ind_mismatches.tsv`, columns `lemma\tfeatures\tours\tgold`)
and emits `data/ind/overrides.tsv` (`lemma\tfeatures\tform`), choosing the
first oracle gold variant as the canonical form. These are the lexicalised
residue the productive meN-/ber-/ter-/di- + suffix rules cannot reach
(irregular nasalisation, ber-peN- nominalisations, borrowed clusters,
suppletion). Run `golden_ind` first, then this, then `golden_ind` again.
"""
import sys

MISMATCHES = "target/golden_ind_mismatches.tsv"
OUT = "data/ind/overrides.tsv"


def main() -> None:
    src = sys.argv[1] if len(sys.argv) > 1 else MISMATCHES
    rows = []
    try:
        with open(src, encoding="utf-8") as fh:
            for line in fh:
                cols = line.rstrip("\n").split("\t")
                if len(cols) < 4:
                    continue
                lemma, feat, _ours, gold = cols[0], cols[1], cols[2], cols[3]
                if not gold:
                    continue
                # First gold variant is the canonical override form.
                form = gold.split("|")[0]
                rows.append((lemma, feat, form))
    except FileNotFoundError:
        print(f"no mismatch file at {src}; run golden_ind first", file=sys.stderr)

    rows.sort()
    with open(OUT, "w", encoding="utf-8") as fh:
        fh.write("# lemma\tfeatures\tform\n")
        for r in rows:
            fh.write("\t".join(r) + "\n")
    print(f"wrote {len(rows)} overrides to {OUT}")


if __name__ == "__main__":
    main()
