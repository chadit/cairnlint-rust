#![allow(dead_code, unused_variables)]

//! Cargo also builds `tests/<name>/main.rs` as its own `--test` crate, so these
//! helpers are test code too.

/// Builds the fixture every suite test starts from.
fn fixture(id: u32) -> Result<(), String> {
    Ok(())
}

#[test]
fn fixture_starts_clean() {
    assert!(fixture(1).is_ok());
}
