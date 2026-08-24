#!/usr/bin/env python3
"""Convert the UniMorph `ind` table into the harness's `lemma\tform\tfeature`
gold, keeping only verb (V;...) rows. The feature is canonicalised to
`V;` + the remaining tokens sorted alphabetically (empty tokens dropped), so
the engine sees one stable key per cell and a would-be second oracle aligns.
"""
import sys


def canon(feat: str) -> str:
    toks = [t for t in feat.split(";") if t != ""]
    if not toks:
        return feat
    return toks[0] + ";" + ";".join(sorted(toks[1:])) if len(toks) > 1 else toks[0]


def main() -> None:
    src = sys.argv[1] if len(sys.argv) > 1 else "data/ind/unimorph_raw.tsv"
    out = sys.argv[2] if len(sys.argv) > 2 else "data/ind/unimorph.tsv"
    seen = set()
    rows = []
    with open(src, encoding="utf-8") as fh:
        for line in fh:
            parts = line.rstrip("\n").split("\t")
            if len(parts) < 3:
                continue
            lemma, form, feat = parts[0], parts[1], parts[2]
            if not feat.startswith("V"):
                continue
            feat = canon(feat)
            key = (lemma, form, feat)
            if key in seen:
                continue
            seen.add(key)
            rows.append((lemma, form, feat))
    rows.sort()
    with open(out, "w", encoding="utf-8") as fh:
        for r in rows:
            fh.write("\t".join(r) + "\n")
    print(f"wrote {len(rows)} verb rows to {out}")


if __name__ == "__main__":
    main()
