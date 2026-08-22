#!/usr/bin/env python3
"""Convert the kaikki.org Tagalog extraction to the UniMorph tgl schema.

kaikki keys each verb entry on an already-focus-marked head (`sumulat`,
`sulatin`, `sulatan`), not on the bare root, and tags the cells by
aspect alone. To line the two oracles up on the *same* key, this adapter
rebuilds UniMorph's view:

  * The **root** is reconstructed from the entry's `tl-infl-*`
    inflection template, whose args 1/2/3 are the onset, the first vowel
    and the remainder (`tl-infl-um` d·a·ting -> dating; `tl-infl-in`
    k·a·in -> kain). This is exactly the segmentation UniMorph lemmatizes
    on.
  * The **trigger** is read from the template: the actor-voice families
    (um, mag, mang, ma, maka, maki, magpa, ...) map to UniMorph AGFOC;
    the patient/locative/benefactive families (in, in-an, i, ipa, ...)
    map to PFOC. A `trigger` arg of `actor`/`total`, when present, wins.
  * The three **aspect** cells come from the clean `tl-verb`
    head-template (arg 2 complete -> PFV, arg 3 progressive -> IPFV, arg
    4 contemplative -> the LGSPEC1 contemplated cell), sidestepping the
    Baybayin/`error-unrecognized-form` noise in the forms list.

Multiple kaikki entries can share a (root, trigger) key (e.g. an -in and
an -an object form): both are emitted, and the harness unions them into
the variant set, so either spelling counts as agreement.

Usage: python3 scripts/tgl/kaikki_to_tsv.py data/tgl/kaikki-tgl.jsonl
"""

import json
import sys

# Inflection-template family -> UniMorph trigger. Actor-voice vs. the
# undergoer voices (object/locative/benefactive/instrument/referential),
# which UniMorph lumps as PFOC.
ACTOR = {
    "um", "mag", "mang", "ma", "maka", "maki", "magka", "magpa",
    "makipag", "magpa-um",
}
PATIENT = {
    "in", "in-an", "i", "isa", "ipa", "ipag", "ika", "pa-in", "pa-an",
    "pag-an", "ka-an", "ma-an",
}


def reconstruct_root(args):
    """Root = onset + first vowel + remainder (template args 1/2/3)."""
    return f"{args.get('1', '')}{args.get('2', '')}{args.get('3', '')}".strip()


def trigger_of(name, args):
    fam = name[len("tl-infl-"):]
    # An explicit trigger word in the template overrides the family guess
    # (tl-infl-ma is actor for mabuhay but object for matira).
    for key in ("4", "5"):
        val = (args.get(key) or "").strip().lower()
        if val in ("actor", "total"):
            return "AGFOC"
        if val in ("object", "locative", "benefactive", "reference",
                   "referential", "instrument", "directional", "causative"):
            return "PFOC"
    if fam in ACTOR:
        return "AGFOC"
    if fam in PATIENT:
        return "PFOC"
    return None


def clean(cell):
    """First surface spelling of a head-template cell (drop Baybayin)."""
    if not cell:
        return ""
    first = cell.split("\n")[0].strip()
    # Baybayin script lives in U+1700..U+171F; a Latin cell has none.
    if any("ᜀ" <= ch <= "ᜟ" for ch in first):
        return ""
    return first if first not in ("+", "-") else ""


def main(path):
    rows = set()
    for line in open(path, encoding="utf-8"):
        d = json.loads(line)
        if d.get("pos") != "verb":
            continue
        its = d.get("inflection_templates") or []
        infl = next((t for t in its
                     if (t.get("name") or "").startswith("tl-infl-")
                     and (t.get("name") or "") != "tl-infl-table"), None)
        if not infl:
            continue
        root = reconstruct_root(infl.get("args", {}))
        trig = trigger_of(infl["name"], infl.get("args", {}))
        if not root or not trig:
            continue
        rows.add((root, root, "V;NFIN"))
        for ht in d.get("head_templates") or []:
            if ht.get("name") != "tl-verb":
                continue
            a = ht.get("args", {})
            for arg, feat in (("2", f"V;PFV;{trig}"),
                              ("3", f"V;IPFV;{trig}"),
                              ("4", f"V;{trig};LGSPEC1")):
                form = clean(a.get(arg))
                if form:
                    rows.add((root, form, feat))
    out = sys.stdout
    for root, form, feat in sorted(rows):
        out.write(f"{root}\t{form}\t{feat}\n")


if __name__ == "__main__":
    main(sys.argv[1])
