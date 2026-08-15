#!/usr/bin/env python3
"""Cross-examine the two gold oracles (UniMorph deu vs kaikki.org) and triage
ablaut's mismatches against their agreement.

Usage: python3 scripts/cross_oracle.py data/unimorph/deu data/kaikki/deu.tsv \
           target/golden_mismatches.tsv

For every (lemma, feature) both oracles cover:
  - agree:      variant sets overlap
  - disagree:   variant sets are disjoint  → oracle data-quality findings
For every ablaut mismatch (from the last golden run):
  - both-against-us: both oracles agree and we differ → real bug candidates
  - oracle-split:    the oracles disagree with each other → adjudication
"""
import collections
import sys


def load(path):
    gold = collections.defaultdict(set)
    for line in open(path):
        parts = line.rstrip("\n").split("\t")
        if len(parts) < 3 or not parts[2].startswith("V"):
            continue
        lemma, form, feat = parts[0].strip(), parts[1].strip(), parts[2].strip()
        if form.startswith("—"):
            continue
        gold[(lemma, feat)].add(form)
    return gold


def main(uni_path, kai_path, mismatch_path):
    uni, kai = load(uni_path), load(kai_path)
    shared = set(uni) & set(kai)
    disjoint = [k for k in shared if not (uni[k] & kai[k])]
    print(f"shared (lemma, feature) slots: {len(shared)}")
    print(f"oracles agree: {len(shared) - len(disjoint)}")
    print(
        f"oracles disagree (disjoint variants): {len(disjoint)}"
        f" ({100 * len(disjoint) / len(shared):.2f}%)"
    )

    both_against_us, oracle_split, single_oracle = [], [], []
    for line in open(mismatch_path):
        lemma, feat, ours, gold = line.rstrip("\n").split("\t")
        key = (lemma, feat)
        if key in uni and key in kai:
            if uni[key] & kai[key]:
                if ours in uni[key] or ours in kai[key]:
                    oracle_split.append((lemma, feat, ours, gold))
                else:
                    both_against_us.append((lemma, feat, ours, gold))
            else:
                oracle_split.append((lemma, feat, ours, gold))
        else:
            single_oracle.append((lemma, feat, ours, gold))

    print(f"\nablaut mismatches from {mismatch_path}:")
    print(f"  both oracles against us (bug candidates): {len(both_against_us)}")
    print(f"  we agree with the other oracle / oracles split: {len(oracle_split)}")
    print(f"  only one oracle covers the slot: {len(single_oracle)}")

    with open("target/bug_candidates.tsv", "w") as f:
        for row in both_against_us:
            f.write("\t".join(row) + "\n")
    print("\nbug candidates written to target/bug_candidates.tsv")
    sample = collections.Counter(l for l, *_ in both_against_us)
    print("worst bug-candidate lemmas:", sample.most_common(12))

    with open("target/oracle_disagreements.tsv", "w") as f:
        for lemma, feat in sorted(disjoint):
            f.write(
                f"{lemma}\t{feat}\t{'|'.join(sorted(uni[(lemma, feat)]))}"
                f"\t{'|'.join(sorted(kai[(lemma, feat)]))}\n"
            )
    print("oracle disagreements written to target/oracle_disagreements.tsv")


if __name__ == "__main__":
    main(*sys.argv[1:4])
