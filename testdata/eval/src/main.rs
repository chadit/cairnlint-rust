//! The bin half of the eval fixture, checked as its own crate beside the lib.

// eval: flag agent_lone_param
fn shout(text: &str, loud: bool) -> String {
    if loud {
        text.to_uppercase()
    } else {
        text.to_owned()
    }
}

fn main() {
    let shouted = [shout("a", true), shout("b", true)];
    let rendered = cairnlint_eval::render("c", false);
    println!("{shouted:?} {rendered}");
}
