# Norwegian Bokmål: two independent oracles (apertium-nob ∩ UniMorph nob)

Bokmål passes the two-oracle criterion cleanly, unlike the EU-24 Slavic
candidates whose kaikki conjugation templates fail to expand. The pair is
independent by lineage:

1. **apertium-nob** — a hand-built lttoolbox dictionary, no Wiktionary
   lineage. `lt-expand` walks it into every `surface:lemma<tags>` pair;
   `scripts/nob/apertium_to_tsv.py` maps the `<vblex>` tags onto the
   shared UniMorph-style bundle (present, preterite, past participle,
   imperative, and the s-form). ~69k verb forms.
2. **UniMorph nob** — the Wiktionary lineage. 14.8k verb forms over 2,224
   lemmas across the same slots (the lemma is the infinitive; there is no
   `V;NFIN` row, and UniMorph's `-` null cells for defective s-forms are
   dropped in `scripts/nob/unimorph_to_tsv.py`).

## Scope and result

Bokmål verbs, like Danish and Swedish, carry no person/number agreement.
The engine (`src/nob.rs`) is the productive class-1 rule — present
`inf+r`, preterite/participle `stem+et`·`stem+a` (both standard
spellings), imperative `stem`, present participle `inf+(e)nde`, s-form
`inf+s` — plus a mined exception table (`data/nob/verbs.tsv`, ~1,345
lemmas) for the class-2 `-te`/`-de` verbs and the strong verbs
(skrive → skreiv·skrev / skrevet).

On the agreement set — 10,186 forms over 2,165 lemmas where both oracles
overlap — the engine scores **100.00%**. Against UniMorph alone (the
local smoke test, which additionally covers the present participle that
apertium files under `<adj><pprs>`) it also scores 100.00% over 12,240
forms.

## Oracle-vs-oracle disagreements

15 slots are excluded from the gold because the oracles disagree; they are
logged with rulings in `disagreements.tsv`. Ten are genuine Bokmål
strong/weak past doublets (slenge → slang · slengte; skvette → skvatt ·
skvetta), and five are one-off oracle artifacts (a UniMorph typo
"oppvuder", two slash-joined UniMorph cells, and apertium
over-regularizing the strong compound gjenvelge → gjenvalgt).
