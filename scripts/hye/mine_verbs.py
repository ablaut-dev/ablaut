#!/usr/bin/env python3
"""Mine the Armenian irregular-verb table (data/hye/verbs.tsv).

Two sources feed it:

1. the two-oracle agreement (UniMorph hye ∩ uniparser), restricted to
   the lemmas the rule engine gets wrong — captured against an empty
   table by scripts/hye/capture_irregulars.sh;
2. scripts/hye/manual.tsv, the core irregulars (գալ, տալ, ուտել, …)
   that UniMorph hye does not list at all, so no agreement exists for
   them.

Every stored column is a principal part: the imperfective converb, the
perfect stem, the aorist stem and its ending set, the two imperatives,
the subject-participle stem, and the passive/causative infinitives.
Columns the rule engine would derive correctly are stored anyway — it
keeps each mined lemma exact and costs a few bytes.

Run after a golden_hye pass; a stable irregulars.txt keeps the mine
reproducible.
"""

import collections

AORIST_I = ["ի", "իր", "", "ինք", "իք", "ին"]
AORIST_A = ["ա", "ար", "ավ", "անք", "աք", "ան"]
AORIST_MIXED = ["ի", "իր", "եց", "ինք", "իք", "ին"]

COLUMNS = ["ipfv", "perf", "aor", "aorset", "imp_sg", "imp_pl", "sub", "pass", "caus"]


def load(path):
    forms = collections.defaultdict(lambda: collections.defaultdict(set))
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            fields = line.rstrip("\n").split("\t")
            if len(fields) != 3:
                continue
            lemma, form, feat = fields
            forms[lemma][feat].add(form)
    return forms


UNIMORPH = load("data/hye/unimorph.tsv")
UNIPARSER = load("data/hye/uniparser.tsv")


def load_rulings():
    """The adjudicated oracle disagreements (docs/hye/disagreements.tsv)."""
    rulings = {}
    with open("docs/hye/disagreements.tsv", encoding="utf-8") as fh:
        for line in fh:
            if line.startswith("#") or not line.strip():
                continue
            cells = line.rstrip("\n").split("\t")
            if len(cells) >= 3:
                rulings[(cells[0], cells[1])] = cells[2]
    return rulings


RULINGS = load_rulings()


def agreed(lemma, feat):
    """The canonical form for a slot, or None.

    A form both oracles carry wins. Where they disagree the ruling in
    docs/hye/disagreements.tsv decides: the named oracle's form, or —
    for a doublet ruled `variant` — the Wiktionary one, which is the
    form the engine standardizes on. An unruled disagreement is not
    mined: the table must not encode one oracle's guess.
    """
    a = UNIMORPH.get(lemma, {}).get(feat, set())
    b = UNIPARSER.get(lemma, {}).get(feat, set())
    both = a & b
    if both:
        return sorted(both)[0]
    if not (a and b):
        return None
    ruling = RULINGS.get((lemma, feat))
    if ruling == "o1":
        return sorted(b)[0]
    if ruling in ("o2", "variant"):
        return sorted(a)[0]
    return None


def strip(form, suffix):
    return form[: -len(suffix)] if form and form.endswith(suffix) else None


def aorist(lemma):
    """The aorist stem and ending set, read off the six aorist forms."""
    slots = [f"V;IND;{n};{p};PST" for n in ("SG", "PL") for p in "123"]
    forms = [agreed(lemma, slot) for slot in slots]
    if any(form is None for form in forms):
        return None, None
    for name, endings in (("1", AORIST_I), ("2", AORIST_A), ("3", AORIST_MIXED)):
        stems = {strip(f, e) if e else f for f, e in zip(forms, endings)}
        if len(stems) == 1 and None not in stems:
            return stems.pop(), name
    return None, None


def mine(lemma):
    row = {c: "-" for c in COLUMNS}
    ipfv = agreed(lemma, "V.CVB;IPFV")
    if ipfv:
        row["ipfv"] = ipfv
    perf = strip(agreed(lemma, "V.CVB;PFV") or "", "ել")
    if perf:
        row["perf"] = perf
    stem, aorset = aorist(lemma)
    if stem:
        row["aor"], row["aorset"] = stem, aorset
    for column, slot in (("imp_sg", "V;IMP;SG;2"), ("imp_pl", "V;IMP;PL;2")):
        form = agreed(lemma, slot)
        if form:
            row[column] = form
    sub = strip(agreed(lemma, "V;V.PTCP;SUB") or "", "ող")
    if sub:
        row["sub"] = sub
    for column, slot in (("pass", "V;PASS"), ("caus", "V;CAUS")):
        form = agreed(lemma, slot)
        if form:
            row[column] = form
    return row


def read_manual():
    rows = {}
    with open("scripts/hye/manual.tsv", encoding="utf-8") as fh:
        for line in fh:
            if line.startswith("#") or not line.strip():
                continue
            cells = line.rstrip("\n").split("\t")
            rows[cells[0]] = dict(zip(COLUMNS, cells[1:]))
    return rows


def main():
    irregulars = set()
    with open("scripts/hye/irregulars.txt", encoding="utf-8") as fh:
        for line in fh:
            line = line.strip()
            if line and not line.startswith("#"):
                irregulars.add(line)
    # Every adjudicated lemma is mined too: its disputed slots are held
    # out of the gold, so the harness never flags it, but the ruling
    # still has to reach the engine.
    irregulars.update(lemma for lemma, _ in RULINGS)

    rows = {lemma: mine(lemma) for lemma in sorted(irregulars)}
    # The hand-supplied core wins over anything mined for the same lemma.
    rows.update(read_manual())

    header = ["# Armenian irregular verbs.",
           "#",
           "# Mined from the UniMorph ∩ uniparser agreement for the lemmas the",
           "# rule engine misses (empty-table capture,",
           "# scripts/hye/capture_irregulars.sh), plus the core irregulars of",
           "# scripts/hye/manual.tsv, which UniMorph hye does not list.",
           "# Regenerate: python3 scripts/hye/mine_verbs.py",
           "#",
           "# aor is the aorist stem; aorset picks its endings — 1 for",
           "# -ի/-իր/-∅/-ինք/-իք/-ին (գրեցի, գրեց), 2 for -ա/-ար/-ավ/-անք/",
           "# -աք/-ան (եկա, եկավ), 3 for the causatives' mixed -ի/-իր/-եց",
           "# (հասցրի, հասցրեց). A `-` cell falls through to the rule.",
           "# lemma\t" + "\t".join(COLUMNS)]
    body = []
    for lemma in sorted(rows):
        row = rows[lemma]
        if all(value == "-" for value in row.values()):
            continue
        body.append(lemma + "\t" + "\t".join(row[column] for column in COLUMNS))
    with open("data/hye/verbs.tsv", "w", encoding="utf-8") as fh:
        fh.write("\n".join(header + body) + "\n")
    print(f"{len(body)} rows written to data/hye/verbs.tsv")


if __name__ == "__main__":
    main()
