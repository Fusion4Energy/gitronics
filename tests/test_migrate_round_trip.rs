//! `migrate` must preserve the model.
//!
//! Splitting a monolithic deck into a gitronics project is only useful if
//! building that project reproduces what went in. The build legitimately
//! reorders cards, rewrites `fill=` into a placeholder and back, and adds a
//! provenance banner, so this compares the model's *content* — the set of cell
//! and surface ids, each cell's material, universe and fill, and the data cards
//! — rather than bytes.

use gitronics::migrate_model;
use migjorn::Model;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

fn parse(path: &Path) -> Model {
    Model::parse(
        &fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display())),
    )
}

fn cell_ids(m: &Model) -> BTreeSet<i64> {
    m.cells().filter_map(|c| c.id()).collect()
}

fn surface_ids(m: &Model) -> BTreeSet<i64> {
    m.surfaces().filter_map(|s| s.id()).collect()
}

/// `(cell id, material, universe, filled universe, starred, transform)` for
/// every cell — the placement facts migrate has to carry through.
type CellFacts = (
    i64,
    Option<i64>,
    Option<i64>,
    Option<i64>,
    bool,
    Option<String>,
);

fn cell_facts(m: &Model) -> Vec<CellFacts> {
    let mut facts: Vec<CellFacts> = m
        .cells()
        .map(|c| {
            let fill = c.fill();
            (
                c.id().unwrap_or_default(),
                c.material(),
                c.universe(),
                fill.as_ref().map(|f| f.universe),
                fill.as_ref().is_some_and(|f| f.starred),
                fill.as_ref().and_then(|f| f.transform.clone()),
            )
        })
        .collect();
    facts.sort();
    facts
}

/// Data cards, normalised for whitespace — `migrate` trims each card's trailing
/// whitespace when it writes them out.
fn data_cards(m: &Model) -> BTreeSet<String> {
    m.data_cards()
        .map(|c| c.text().split_whitespace().collect::<Vec<_>>().join(" "))
        .collect()
}

#[test]
fn migrate_then_build_reproduces_the_original_model() {
    let original_path = Path::new("resources/filled_model.mcnp");
    let project = tempdir().unwrap();

    migrate_model(original_path, project.path()).expect("migration failed");

    let original = parse(original_path);
    let rebuilt = parse(&project.path().join("output/assembled.mcnp"));

    assert_eq!(cell_ids(&original), cell_ids(&rebuilt), "cell ids differ");
    assert_eq!(
        surface_ids(&original),
        surface_ids(&rebuilt),
        "surface ids differ"
    );
    assert_eq!(
        cell_facts(&original),
        cell_facts(&rebuilt),
        "a cell's material, universe or fill placement changed"
    );
    assert_eq!(
        data_cards(&original),
        data_cards(&rebuilt),
        "data cards differ"
    );
    assert!(
        rebuilt.validate().is_empty(),
        "rebuilt model does not validate: {:?}",
        rebuilt.validate()
    );
}

/// The fixture has to actually exercise migration, or the test above is
/// vacuous — which is exactly what `resources/simple_model.mcnp` (no `fill=`
/// cards at all) made the original migrate test.
#[test]
fn the_round_trip_fixture_exercises_fills() {
    let original = parse(Path::new("resources/filled_model.mcnp"));
    let fills: Vec<_> = original.cells().filter_map(|c| c.fill()).collect();

    assert!(fills.len() >= 3, "fixture needs several filled cells");
    assert!(
        fills.iter().any(|f| f.transform.is_none()),
        "fixture needs a bare fill"
    );
    assert!(
        fills.iter().any(|f| f.transform.is_some() && !f.starred),
        "fixture needs a fill with a transformation"
    );
    assert!(
        fills.iter().any(|f| f.starred),
        "fixture needs a starred fill"
    );
}
