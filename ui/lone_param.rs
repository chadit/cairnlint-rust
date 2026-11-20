#![allow(dead_code, unused_variables)]

const PAGE_SIZE: usize = 10;

enum Mode {
    Strict,
    Loose,
}

fn render(template: &str, strict: bool) -> String {
    if strict {
        format!("strict:{template}")
    } else {
        template.to_owned()
    }
}

fn format_mode(template: &str, mode: &str) -> String {
    format!("{mode}:{template}")
}

fn once(template: &str, strict: bool) -> String {
    if strict {
        template.to_uppercase()
    } else {
        template.to_owned()
    }
}

fn escaping(template: &str, strict: bool) -> String {
    if strict {
        template.to_uppercase()
    } else {
        template.to_owned()
    }
}

fn take(items: &[u8], limit: usize) -> &[u8] {
    &items[..limit.min(items.len())]
}

fn run(mode: Mode, retries: u32) -> u32 {
    match mode {
        Mode::Strict => retries,
        Mode::Loose => 0,
    }
}

fn lookup(key: &str, fallback: Option<u32>) -> Option<u32> {
    key.parse().ok().or(fallback)
}

fn log(_level: u8, message: &str) -> usize {
    message.len()
}

struct Renderer {
    scale: u32,
}

impl Renderer {
    fn draw(&self, width: u32, bold: bool) -> u32 {
        if bold { width * self.scale } else { width }
    }
}

trait Shape {
    fn area(&self, scale: u32) -> u32;

    fn scaled(&self, factor: u32) -> u32 {
        self.area(1) * factor
    }
}

struct Square;

struct Circle;

impl Shape for Square {
    fn area(&self, scale: u32) -> u32 {
        scale * 4
    }
}

impl Shape for Circle {
    fn area(&self, scale: u32) -> u32 {
        scale * 3
    }
}

macro_rules! generated_fn {
    () => {
        fn generated(flag: bool) -> bool {
            !flag
        }
    };
}

generated_fn!();

fn main() {
    let items = [1, 2, 3];
    let renderer = Renderer { scale: 2 };
    let rendered = [
        render("page", true),
        render("header", true),
        render("footer", true),
    ];
    let modes = [format_mode("page", "wide"), format_mode("header", "narrow")];
    let single = once("page", true);
    let escaped = [escaping("page", true), escaping("header", true)];
    let formatter: fn(&str, bool) -> String = escaping;
    let taken = [take(&items, PAGE_SIZE), take(&items, 10)];
    let ran = [run(Mode::Strict, 1), run(Mode::Strict, 2)];
    let found = [lookup("a", None), lookup("b", None)];
    let logged = [log(1, "a"), log(1, "b")];
    let drawn = [renderer.draw(10, true), Renderer::draw(&renderer, 20, true)];
    let areas = [Square.area(2), Square.area(2), Circle.area(2)];
    let scaled = [Square.scaled(3), Circle.scaled(3)];
    let flags = [generated(true), generated(true)];
}

// SAFETY: the declaration matches libc's `abs`, which the platform C library
// always provides.
unsafe extern "C" {
    fn abs(input: i32) -> i32;
}

fn call_foreign() -> [i32; 2] {
    // SAFETY: libc's abs is defined for every i32 except i32::MIN, and the
    // fixture passes -1.
    unsafe { [abs(-1), abs(-1)] }
}

#[unsafe(no_mangle)]
extern "C" fn exported_sum(value: u32, offset: u32) -> u32 {
    value + offset
}

fn call_exported() -> [u32; 2] {
    [exported_sum(1, 4), exported_sum(2, 4)]
}

fn place((x, y): (u32, u32), flag: bool) -> u32 {
    if flag { x + y } else { x }
}

fn call_place() -> [u32; 2] {
    [place((1, 2), true), place((1, 2), true)]
}
