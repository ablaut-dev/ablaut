//! Python bindings (feature `python`, built with maturin):
//! `pip install ablaut` → `ablaut.conjugation_table("aufstehen")`.

use crate::table::Table;
use crate::Verb;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;

fn table_to_dict<'py>(py: Python<'py>, t: &Table) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("infinitive", &t.infinitive)?;
    d.set_item("zu_infinitive", &t.zu_infinitive)?;
    d.set_item("perfect_infinitive", &t.perfect_infinitive)?;
    d.set_item("auxiliary", &t.auxiliary)?;
    d.set_item("present_participle", &t.present_participle)?;
    d.set_item("past_participle", &t.past_participle)?;
    d.set_item("imperative", &t.imperative)?;
    d.set_item("imperative_extended", &t.imperative_extended)?;
    d.set_item("present", &t.present)?;
    d.set_item("preterite", &t.preterite)?;
    d.set_item("konjunktiv1", &t.konjunktiv1)?;
    d.set_item("konjunktiv2", &t.konjunktiv2)?;
    d.set_item("perfect", &t.perfect)?;
    d.set_item("pluperfect", &t.pluperfect)?;
    d.set_item("future1", &t.future1)?;
    d.set_item("future2", &t.future2)?;
    d.set_item("wuerde", &t.wuerde)?;
    d.set_item("konj1_perfect", &t.konj1_perfect)?;
    d.set_item("konj1_future", &t.konj1_future)?;
    d.set_item("konj2_pluperfect", &t.konj2_pluperfect)?;
    d.set_item("konj2_future2", &t.konj2_future2)?;
    Ok(d)
}

/// The full conjugation table for an infinitive, as a dict. Rows are
/// [ich, du, er/sie/es, wir, ihr, sie]; imperatives are None for modals.
#[pyfunction]
fn conjugation_table<'py>(py: Python<'py>, infinitive: &str) -> PyResult<Bound<'py, PyDict>> {
    let v = Verb::from_infinitive(infinitive).map_err(|e| PyValueError::new_err(e.to_string()))?;
    table_to_dict(py, &Table::build(&v))
}

#[pymodule]
fn ablaut(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(conjugation_table, m)?)?;
    Ok(())
}
