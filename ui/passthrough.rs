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

impl Inner {
    fn combine(&self, left: &str, right: &str) -> String {
        format!("{left}{}{right}", self.sep)
    }
}

struct Swapped {
    inner: Inner,
}

impl Swapped {
    fn combine(&self, left: &str, right: &str) -> String {
        self.inner.combine(right, left)
    }
}

struct Delegating {
    inner: Inner,
}

impl Delegating {
    fn sep(&self) -> &str {
        self.label()
    }

    fn label(&self) -> &str {
        &self.inner.sep
    }
}

struct Stack<T> {
    items: Vec<T>,
}

impl<T> Stack<T> {
    fn len(&self) -> usize {
        self.items.len()
    }
}

struct Spelled {
    inner: Inner,
}

impl Spelled {
    fn join(&self, parts: &[String]) -> String {
        Inner::join(&self.inner, parts)
    }
}

struct SpelledMixed {
    inner: Inner,
    seen: Cell<u32>,
}

impl SpelledMixed {
    fn combine(&self, left: &str, right: &str) -> String {
        Inner::combine(&self.inner, left, right)
    }

    fn count(&self) -> u32 {
        self.seen.set(self.seen.get() + 1);
        self.seen.get()
    }
}

fn join_with(inner: &Inner, parts: &[String]) -> String {
    inner.join(parts)
}

struct Helped {
    inner: Inner,
}

impl Helped {
    fn join(&self, parts: &[String]) -> String {
        join_with(&self.inner, parts)
    }
}

struct SpelledSwapped {
    inner: Inner,
}

impl SpelledSwapped {
    fn combine(&self, left: &str, right: &str) -> String {
        Inner::combine(&self.inner, right, left)
    }
}

struct SpelledBoxed {
    inner: Box<Inner>,
}

impl SpelledBoxed {
    fn join(&self, parts: &[String]) -> String {
        Inner::join(&*self.inner, parts)
    }
}

fn fetch_inner() -> Box<Inner> {
    Box::new(Inner { sep: String::new() })
}

struct Fetched {
    inner: Box<Inner>,
}

impl Fetched {
    fn join(&self, parts: &[String]) -> String {
        Inner::join(&*fetch_inner(), parts)
    }
}
