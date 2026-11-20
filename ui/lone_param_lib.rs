#![crate_type = "lib"]
#![allow(dead_code, unused_variables)]

pub fn exported(template: &str, strict: bool) -> String {
    if strict {
        template.to_uppercase()
    } else {
        template.to_owned()
    }
}

fn private(template: &str, strict: bool) -> String {
    if strict {
        template.to_uppercase()
    } else {
        template.to_owned()
    }
}

pub fn entry() -> [String; 4] {
    [
        exported("a", true),
        exported("b", true),
        private("c", true),
        private("d", true),
    ]
}
