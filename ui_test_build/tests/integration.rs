#![allow(dead_code, unused_variables)]

//! Cargo builds each `tests/*.rs` file as its own `--test` crate, where only
//! the `#[test]` functions carry a test marker. The helpers below are test code.

/// Builds the fixture every integration test starts from.
fn fixture(id: u32) -> Result<(), String> {
    Ok(())
}

fn label(name: &str, strict: bool) -> String {
    if strict {
        name.to_uppercase()
    } else {
        name.to_owned()
    }
}

struct Holder {
    inner: Vec<u8>,
}

impl Holder {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

#[test]
fn fixture_starts_clean() {
    assert!(fixture(1).is_ok());
    assert_eq!(label("a", true), "A");
}

#[test]
fn labels_strictly() {
    assert_eq!(label("b", true), "B");
}
