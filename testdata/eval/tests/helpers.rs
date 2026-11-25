//! An integration test crate, where every item is test code.
#![allow(unused_variables)]

/// Builds the fixture every test starts from.
// eval: clean agent_stub_body
fn fixture(id: u32) -> Result<(), String> {
    Ok(())
}

#[test]
fn fixture_starts_clean() {
    assert!(fixture(1).is_ok());
}
