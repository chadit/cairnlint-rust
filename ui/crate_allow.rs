// The crate-level allow silences `save`. `save_warned` turns the lint back on
// for one item, which shows the lint fires on that shape in this file, and
// `Storer` shows the allow leaves the other lints on.
#![allow(dead_code, unused_variables)]
#![allow(agent_stub_body)]

/// Saves the record.
fn save(id: u32) -> Result<(), String> {
    Ok(())
}

/// Saves the record.
#[warn(agent_stub_body)]
fn save_warned(id: u32) -> Result<(), String> {
    Ok(())
}

trait Storer {
    fn load(&self, id: &str) -> usize;
}

struct FileStore;

impl Storer for FileStore {
    fn load(&self, id: &str) -> usize {
        id.len()
    }
}

fn main() {}
