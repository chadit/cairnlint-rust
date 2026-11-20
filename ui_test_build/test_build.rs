#![allow(dead_code, unused_variables)]

trait Storer {
    fn load(&self, id: &str) -> usize;
}

struct FileStore;

impl Storer for FileStore {
    fn load(&self, id: &str) -> usize {
        id.len()
    }
}

trait Clock {
    fn now(&self) -> u64;
}

struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> u64 {
        u64::from(std::process::id())
    }
}

fn render(template: &str, strict: bool) -> String {
    if strict {
        template.to_uppercase()
    } else {
        template.to_owned()
    }
}

fn production() -> [String; 2] {
    [render("a", true), render("b", true)]
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeStore;

    impl Storer for FakeStore {
        fn load(&self, id: &str) -> usize {
            0
        }
    }

    /// Builds the fixture store every test starts from.
    fn fixture(id: u32) -> Result<(), String> {
        Ok(())
    }

    struct Recorder {
        inner: FileStore,
    }

    impl Recorder {
        fn load(&self, id: &str) -> usize {
            self.inner.load(id)
        }
    }

    fn titled(name: &str, strict: bool) -> String {
        if strict {
            name.to_uppercase()
        } else {
            name.to_owned()
        }
    }

    #[test]
    fn renders_loose_templates() {
        assert_eq!(render("a", false), "a");
    }

    #[test]
    fn titles_strictly() {
        assert_eq!(titled("a", true), "A");
    }

    #[test]
    fn titles_empty_names() {
        assert_eq!(titled("", true), "");
    }
}
