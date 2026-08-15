//! Fully suppletive verbs: *sein*, *werden*, *tun*. Their paradigms resist
//! stem+ending decomposition (bin/bist/ist; wirst without -d-), so every
//! synthetic form is stored outright.

use crate::Auxiliary;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Suppletive {
    pub infinitive: &'static str,
    /// Six-slot rows: 1sg, 2sg, 3sg, 1pl, 2pl, 3pl.
    pub present: [&'static str; 6],
    pub preterite: [&'static str; 6],
    pub konjunktiv1: [&'static str; 6],
    pub konjunktiv2: [&'static str; 6],
    pub part1: &'static str,
    pub part2: &'static str,
    pub imp_sg: &'static str,
    pub imp_pl: &'static str,
    pub aux: Auxiliary,
}

pub(crate) static SEIN: Suppletive = Suppletive {
    infinitive: "sein",
    present: ["bin", "bist", "ist", "sind", "seid", "sind"],
    preterite: ["war", "warst", "war", "waren", "wart", "waren"],
    konjunktiv1: ["sei", "seist", "sei", "seien", "seiet", "seien"],
    konjunktiv2: ["wäre", "wärest", "wäre", "wären", "wäret", "wären"],
    part1: "seiend",
    part2: "gewesen",
    imp_sg: "sei",
    imp_pl: "seid",
    aux: Auxiliary::Sein,
};

pub(crate) static WERDEN: Suppletive = Suppletive {
    infinitive: "werden",
    present: ["werde", "wirst", "wird", "werden", "werdet", "werden"],
    preterite: ["wurde", "wurdest", "wurde", "wurden", "wurdet", "wurden"],
    konjunktiv1: ["werde", "werdest", "werde", "werden", "werdet", "werden"],
    konjunktiv2: ["würde", "würdest", "würde", "würden", "würdet", "würden"],
    part1: "werdend",
    part2: "geworden",
    imp_sg: "werde",
    imp_pl: "werdet",
    aux: Auxiliary::Sein,
};

pub(crate) static TUN: Suppletive = Suppletive {
    infinitive: "tun",
    present: ["tue", "tust", "tut", "tun", "tut", "tun"],
    preterite: ["tat", "tatest", "tat", "taten", "tatet", "taten"],
    konjunktiv1: ["tue", "tuest", "tue", "tuen", "tuet", "tuen"],
    konjunktiv2: ["täte", "tätest", "täte", "täten", "tätet", "täten"],
    part1: "tuend",
    part2: "getan",
    imp_sg: "tu",
    imp_pl: "tut",
    aux: Auxiliary::Haben,
};

pub(crate) fn lookup(infinitive: &str) -> Option<&'static Suppletive> {
    match infinitive {
        "sein" => Some(&SEIN),
        "werden" => Some(&WERDEN),
        "tun" => Some(&TUN),
        _ => None,
    }
}
