#![allow(dead_code, unused_variables)]

use std::cell::Cell;
use std::fmt;

struct Inner {
    sep: String,
}

impl Inner {
    fn join(&self, parts: &[String]) -> String {
        parts.join(&self.sep)
    }

    fn sep(&self) -> &str {
        &self.sep
    }
}

struct Wrapper {
    inner: Inner,
}

impl Wrapper {
    fn new(sep: &str) -> Self {
        Self {
            inner: Inner {
                sep: sep.to_owned(),
            },
        }
    }

    fn join(&self, parts: &[String]) -> String {
        self.inner.join(parts)
    }

    fn sep(&self) -> &str {
        self.inner.sep()
    }
}

impl fmt::Display for Wrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.sep.fmt(f)
    }
}

struct Newtype(Inner);

impl Newtype {
    fn join(&self, parts: &[String]) -> String {
        self.0.join(parts)
    }
}

struct Bag {
    items: Vec<u8>,
}

impl Bag {
    fn clear(&mut self) {
        self.items.clear();
    }
}

struct Mixed {
    inner: Inner,
    seen: Cell<u32>,
}

impl Mixed {
    fn join(&self, parts: &[String]) -> String {
        self.inner.join(parts)
    }

    fn count(&self) -> u32 {
        self.seen.set(self.seen.get() + 1);
        self.seen.get()
    }
}

struct Pair {
    left: Inner,
    right: Inner,
}

impl Pair {
    fn join_left(&self, parts: &[String]) -> String {
        self.left.join(parts)
    }
}

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

struct Reordered {
    inner: Inner,
}

impl Reordered {
    fn join(&self, parts: &[String]) -> String {
        self.inner.join(&parts[1..])
    }
}

struct Forwarded {
    inner: Inner,
}

impl fmt::Display for Forwarded {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.sep.fmt(f)
    }
}

struct Later {
    inner: Inner,
}

impl Later {
    async fn join(&self, parts: &[String]) -> String {
        self.inner.join(parts)
    }
}

macro_rules! generated_forwarder {
    ($name:ident) => {
        struct $name {
            inner: Inner,
        }

        impl $name {
            fn join(&self, parts: &[String]) -> String {
                self.inner.join(parts)
            }
        }
    };
}

generated_forwarder!(Generated);

macro_rules! forward_len {
    () => {
        fn len(&self) -> usize {
            self.inner.len()
        }
    };
}

struct Mac {
    inner: Vec<u8>,
}

impl Mac {
    forward_len!();

    fn first(&self) -> Option<&u8> {
        self.inner.first()
    }
}

struct Boxed {
    inner: Vec<u8>,
}

impl Boxed {
    fn len(&self) -> usize {
        self.inner.len()
    }

    fn into_inner(self: Box<Self>) -> Vec<u8> {
        self.inner
    }
}

struct Returning {
    inner: Inner,
}

impl Returning {
    fn join(&self, parts: &[String]) -> String {
        return self.inner.join(parts);
    }
}

struct Nested {
    outer: Wrapper,
    label: String,
}

impl Nested {
    fn join(&self, parts: &[String]) -> String {
        self.outer.inner.join(parts)
    }
}

fn main() {}
