//! Input for the eval in `tests/eval.rs` at the repository root. A
//! `// eval: flag <lint>` line sits above each line a lint must report, and an
//! `// eval: clean <lint>` line above a near miss it must leave alone.
#![allow(dead_code, unused_variables)]

// eval: flag agent_lone_impl
trait Storer {
    fn load(&self, id: &str) -> usize;
}

struct FileStore;

impl Storer for FileStore {
    fn load(&self, id: &str) -> usize {
        id.len()
    }
}

// eval: clean agent_lone_impl
trait Notifier {
    fn notify(&self, message: &str) -> bool;
}

struct Email;

struct Sms;

impl Notifier for Email {
    fn notify(&self, message: &str) -> bool {
        message.contains('@')
    }
}

impl Notifier for Sms {
    fn notify(&self, message: &str) -> bool {
        message.len() < 160
    }
}

/// Saves the record for `id`.
// eval: flag agent_stub_body
fn save(id: u32) -> Result<(), String> {
    Ok(())
}

/// Returns the sum of `a` and `b`.
// eval: clean agent_stub_body
fn add(a: i32, b: i32) -> i32 {
    a + b
}

struct Inner {
    sep: String,
}

impl Inner {
    fn join(&self, parts: &[String]) -> String {
        parts.join(&self.sep)
    }
}

// eval: flag agent_passthrough
struct Wrapper {
    inner: Inner,
}

impl Wrapper {
    fn join(&self, parts: &[String]) -> String {
        self.inner.join(parts)
    }
}

// eval: clean agent_passthrough
struct Guarded {
    inner: Inner,
}

impl Guarded {
    fn join(&self, parts: &[String]) -> String {
        if parts.is_empty() {
            return String::new();
        }
        self.inner.join(parts)
    }
}

// Other crates, such as this package's bin, can call `render`, so the two
// calls in `pages` are not every call site.
// eval: clean agent_lone_param
pub fn render(template: &str, strict: bool) -> String {
    if strict {
        template.to_uppercase()
    } else {
        template.to_owned()
    }
}

pub fn pages() -> [String; 2] {
    [render("a", true), render("b", true)]
}

// The plain build sees one implementation, and the test build sees the
// `FakeClock` double too. lint.sh keeps a finding only when every build
// reports it, so this trait is not reported.
// eval: clean agent_lone_impl
trait Clock {
    fn now(&self) -> u64;
}

struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> u64 {
        u64::from(std::process::id())
    }
}

// Production passes `true` at both call sites, and only a test passes
// `false`. The test build sees that value, so the parameter is not reported.
// eval: clean agent_lone_param
fn title(name: &str, strict: bool) -> String {
    if strict {
        name.to_uppercase()
    } else {
        name.to_owned()
    }
}

pub fn titles() -> [String; 2] {
    [title("a", true), title("b", true)]
}

#[cfg(test)]
mod tests {
    use super::Clock;

    // eval: clean agent_stub_body
    fn pending() -> u32 {
        todo!()
    }

    struct FakeClock;

    impl Clock for FakeClock {
        fn now(&self) -> u64 {
            7
        }
    }

    #[test]
    fn pages_are_strict() {
        assert_eq!(super::pages(), ["A", "B"]);
    }

    #[test]
    fn fake_clock_is_fixed() {
        assert_eq!(FakeClock.now(), 7);
    }

    #[test]
    fn loose_titles_keep_case() {
        assert_eq!(super::title("a", false), "a");
    }
}
