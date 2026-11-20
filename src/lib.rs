//! Agent-tier lints for Rust: heuristics that catch the code an AI model tends
//! to write. They trade precision for recall on purpose, because an LLM triages
//! the output. Every lint name starts with `agent_`, so a caller can keep these
//! diagnostics apart from rustc and clippy output by name alone.
// rustc's `declare_lint!` expands to a struct literal that leaves fields to
// their defaults, so every crate calling it needs `default_field_values`.
#![feature(default_field_values, rustc_private)]

extern crate rustc_ast;
extern crate rustc_attr_ir;
extern crate rustc_data_structures;
extern crate rustc_hir;
extern crate rustc_lint;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

mod block;
mod lone_impl;
mod lone_param;
mod passthrough;
mod stub_body;
mod test_code;

dylint_linting::dylint_library!();

/// Entry point Dylint calls after loading the library.
// SAFETY: Dylint finds this symbol by name, this cdylib defines it once, and
// Dylint loads the library with the toolchain pinned in rust-toolchain.toml, so
// the Rust-ABI signature matches.
#[unsafe(no_mangle)]
pub fn register_lints(_sess: &rustc_session::Session, lint_store: &mut rustc_lint::LintStore) {
    lint_store.register_lints(&[
        stub_body::AGENT_STUB_BODY,
        lone_impl::AGENT_LONE_IMPL,
        passthrough::AGENT_PASSTHROUGH,
        lone_param::AGENT_LONE_PARAM,
    ]);
    lint_store.register_late_lint_pass(Box::new(|_| Box::new(stub_body::StubBody)));
    lint_store.register_late_lint_pass(Box::new(|_| Box::new(lone_impl::LoneImpl)));
    lint_store.register_late_lint_pass(Box::new(|_| Box::<passthrough::Passthrough>::default()));
    lint_store.register_late_lint_pass(Box::new(|_| Box::<lone_param::LoneParam>::default()));
}

/// `#[cfg(test)]` items exist only when tests are compiled, so the test double
/// and test-code exemptions need a `--test` build of their own. Both suites run
/// in one test because `dylint_testing` holds a shared lock, and a panic in one
/// test would surface in the other as a `PoisonError`.
#[test]
fn ui_fixtures_emit_expected_diagnostics() {
    dylint_testing::ui::Test::src_base(env!("CARGO_PKG_NAME"), "ui")
        .rustc_flags(["--edition=2024"])
        .run();
    dylint_testing::ui::Test::src_base(env!("CARGO_PKG_NAME"), "ui_test_build")
        .rustc_flags(["--edition=2024", "--test"])
        .run();
}
