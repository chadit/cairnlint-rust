#![allow(dead_code, unused_variables)]

/// Saves the record.
#[allow(agent_stub_body)]
fn save(id: u32) -> Result<(), String> {
    Ok(())
}

/// Loads the record.
#[expect(agent_stub_body)]
fn load(id: u32) -> Result<(), String> {
    Ok(())
}

#[cfg_attr(dylint_lib = "cairnlint_rust", allow(agent_lone_impl))]
trait Storer {
    fn load(&self, id: &str) -> usize;
}

struct FileStore;

impl Storer for FileStore {
    fn load(&self, id: &str) -> usize {
        id.len()
    }
}

struct Inner;

impl Inner {
    fn size(&self, extra: usize) -> usize {
        extra + 1
    }
}

#[allow(agent_passthrough)]
struct Wrapper {
    inner: Inner,
}

impl Wrapper {
    fn size(&self, extra: usize) -> usize {
        self.inner.size(extra)
    }
}

struct Mixed {
    inner: Inner,
    count: usize,
}

impl Mixed {
    #[allow(agent_passthrough)]
    fn size(&self, extra: usize) -> usize {
        self.inner.size(extra)
    }

    fn count(&self) -> usize {
        self.count + 1
    }
}

#[allow(agent_lone_param)]
fn render(template: &str, strict: bool) -> String {
    if strict {
        template.to_uppercase()
    } else {
        template.to_owned()
    }
}

fn main() {
    let rendered = [render("a", true), render("b", true)];
}

struct AllowedMethods {
    inner: Inner,
}

impl AllowedMethods {
    #[allow(agent_passthrough)]
    fn size(&self, extra: usize) -> usize {
        self.inner.size(extra)
    }

    #[allow(agent_passthrough)]
    fn bigger(&self, extra: usize) -> usize {
        self.inner.size(extra)
    }
}

struct AllowedBlock {
    inner: Inner,
}

#[allow(agent_passthrough)]
impl AllowedBlock {
    fn size(&self, extra: usize) -> usize {
        self.inner.size(extra)
    }

    fn bigger(&self, extra: usize) -> usize {
        self.inner.size(extra)
    }
}

struct ExpectedMethod {
    inner: Inner,
}

impl ExpectedMethod {
    #[expect(agent_passthrough)]
    fn size(&self, extra: usize) -> usize {
        self.inner.size(extra)
    }
}
