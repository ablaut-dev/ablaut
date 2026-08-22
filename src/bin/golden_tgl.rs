//! Tagalog golden-test harness: score the engine against the agreement
//! of the two Tagalog oracles (UniMorph tgl and the kaikki.org Tagalog
//! extraction, re-keyed onto UniMorph's root+trigger schema).
//!
//! Usage: cargo run --release --bin golden_tgl [gold.tsv ...] [--check]
//!        (default: data/tgl/kaikki.tsv data/tgl/unimorph.tsv —
//!         see scripts/tgl/fetch_unimorph.sh, scripts/tgl/kaikki_to_tsv.py)
//!
//! With two gold files only the slots the oracles agree on are scored;
//! the slots where they contradict each other are the disagreement
//! corpus (which trigger a root lexicalizes, an -in vs -an undergoer).

use ablaut::harness::{run, Spec};
use ablaut::tgl::{Aspect, Focus, Verb};

const CATEGORIES: [&str; 3] = ["root", "actor", "patient"];

fn cell(verb: &Verb, focus: Focus, aspect: Aspect) -> Option<Vec<String>> {
    let forms = verb.forms(focus, aspect);
    (!forms.is_empty()).then_some(forms)
}

fn generate(verb: &Verb, features: &str) -> Option<Vec<String>> {
    use Aspect::{Contemplated as C, Imperfective as I, Perfective as P};
    use Focus::{Actor as AG, Patient as PF};
    match features {
        "V;NFIN" => Some(vec![verb.root().to_string()]),
        "V;PFV;AGFOC" => cell(verb, AG, P),
        "V;IPFV;AGFOC" => cell(verb, AG, I),
        "V;AGFOC;LGSPEC1" => cell(verb, AG, C),
        "V;PFV;PFOC" => cell(verb, PF, P),
        "V;IPFV;PFOC" => cell(verb, PF, I),
        "V;PFOC;LGSPEC1" => cell(verb, PF, C),
        _ => None,
    }
}

fn category(features: &str) -> &'static str {
    if features == "V;NFIN" {
        "root"
    } else if features.contains("AGFOC") {
        "actor"
    } else {
        "patient"
    }
}

fn main() {
    run(Spec {
        lang: "tgl",
        default_paths: ["data/tgl/kaikki.tsv", "data/tgl/unimorph.tsv"],
        adjudications: "docs/tgl/adjudications.tsv",
        mismatches: "target/golden_tgl_mismatches.tsv",
        categories: &CATEGORIES,
        min_form_pct: 98.0,
        min_lemma_coverage_pct: 99.0,
        carry_features: &[],
        parse: |lemma| Verb::from_infinitive(lemma).ok(),
        generate,
        category,
    });
}
