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

pub(crate) fn crate_only(template: &str, strict: bool) -> String {
    if strict {
        template.to_uppercase()
    } else {
        template.to_owned()
    }
}

pub fn entry_crate_only() -> [String; 2] {
    [crate_only("e", true), crate_only("f", true)]
}
