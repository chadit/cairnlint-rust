#![allow(dead_code, unused_variables)]

// Each `#[allow]` case has an unsuppressed twin with the same shape, named
// `*Twin` or `*_twin`. The twin's warning shows the lint fires on that shape,
// so the allowed copy's silence comes from the attribute.

/// Saves the record.
#[allow(agent_stub_body)]
fn save(id: u32) -> Result<(), String> {
    Ok(())
}

/// Saves the record.
fn save_twin(id: u32) -> Result<(), String> {
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

trait StorerTwin {
    fn load(&self, id: &str) -> usize;
}

impl StorerTwin for FileStore {
    fn load(&self, id: &str) -> usize {
        id.len()
    }
}

#[expect(agent_lone_impl)]
trait Indexer {
    fn index(&self, id: &str) -> usize;
}

impl Indexer for FileStore {
    fn index(&self, id: &str) -> usize {
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

struct WrapperTwin {
    inner: Inner,
}

impl WrapperTwin {
    fn size(&self, extra: usize) -> usize {
        self.inner.size(extra)
    }
}

#[expect(agent_passthrough)]
struct ExpectedWrapper {
    inner: Inner,
}

impl ExpectedWrapper {
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

struct MixedTwin {
    inner: Inner,
    count: usize,
}

impl MixedTwin {
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

fn render_twin(template: &str, strict: bool) -> String {
    if strict {
        template.to_uppercase()
    } else {
        template.to_owned()
    }
}

#[expect(agent_lone_param)]
fn render_expected(template: &str, strict: bool) -> String {
    if strict {
        template.to_uppercase()
    } else {
        template.to_owned()
    }
}

// Allowing `agent_lone_param` here leaves `agent_stub_body` on, so the canned
// body is still reported.
/// Stores the record.
#[allow(agent_lone_param)]
fn store(id: u32, strict: bool) -> Result<(), String> {
    Ok(())
}

fn main() {
    let rendered = [render("a", true), render("b", true)];
    let rendered_twin = [render_twin("a", true), render_twin("b", true)];
    let expected = [render_expected("a", true), render_expected("b", true)];
    let stored = [store(1, true), store(2, true)];
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

// The twin of both `AllowedMethods` and `AllowedBlock`.
struct AllowedTwin {
    inner: Inner,
}

impl AllowedTwin {
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
