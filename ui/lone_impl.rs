#![allow(dead_code, unused_variables)]

use std::fmt::Debug;

struct FileStore {
    root: String,
}

trait Storer {
    fn load(&self, id: &str) -> Vec<u8>;
}

impl Storer for FileStore {
    fn load(&self, id: &str) -> Vec<u8> {
        format!("{}/{id}", self.root).into_bytes()
    }
}

trait Slot<T> {
    fn put(&self, value: T) -> usize;
}

impl Slot<u8> for FileStore {
    fn put(&self, value: u8) -> usize {
        usize::from(value) + self.root.len()
    }
}

impl Slot<u16> for FileStore {
    fn put(&self, value: u16) -> usize {
        usize::from(value) + self.root.len()
    }
}

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

trait Marker {}

impl Marker for FileStore {}

trait Describe {
    fn describe(&self) -> String;
}

impl<T: Debug> Describe for T {
    fn describe(&self) -> String {
        format!("{self:?}")
    }
}

struct Opaque;

impl Describe for Opaque {
    fn describe(&self) -> String {
        String::from("opaque")
    }
}

trait ByRef {
    fn id(&self) -> u8;
}

impl ByRef for &Email {
    fn id(&self) -> u8 {
        1
    }
}

impl ByRef for &Sms {
    fn id(&self) -> u8 {
        2
    }
}

struct Holder<T> {
    value: T,
}

trait Held {
    fn size(&self) -> usize;
}

impl Held for Holder<u8> {
    fn size(&self) -> usize {
        usize::from(self.value)
    }
}

impl Held for Holder<String> {
    fn size(&self) -> usize {
        self.value.len()
    }
}

impl Held for Box<Email> {
    fn size(&self) -> usize {
        1
    }
}

impl Held for Box<Sms> {
    fn size(&self) -> usize {
        2
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

#[cfg(test)]
struct FakeClock;

#[cfg(test)]
impl Clock for FakeClock {
    fn now(&self) -> u64 {
        7
    }
}

trait Unimplemented {
    fn reticulate(&self, splines: u32) -> bool;
}

macro_rules! generated_trait {
    () => {
        trait Generated {
            fn run(&self) -> bool;
        }

        impl Generated for Email {
            fn run(&self) -> bool {
                self.notify("generated@example.com")
            }
        }
    };
}

generated_trait!();

fn main() {}
