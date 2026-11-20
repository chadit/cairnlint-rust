//! `agent_lone_impl` flags a local trait that exactly one type in the crate
//! implements. A trait written for one implementation carries a generality
//! nothing uses: the concrete type can go into the signature instead, which
//! removes the trait and the indirection at its uses.
//!
//! The count covers the compilation being checked. A test double under
//! `#[cfg(test)]` is visible only when tests are compiled, so a plain
//! `cargo dylint` run reports a trait that such a double would clear; run with
//! `-- --all-targets` and read the test build's verdict.
//!
//! Not reported: a trait with no items (a marker, or a supertrait bundle), a
//! trait with a blanket impl (`impl<T: Bound> Trait for T`, which covers an
//! open set of types), a trait with no implementation, and a trait that macro
//! output declares.
//!
//! Known problems: a public trait in a library is an extension point for
//! downstream crates, whose impls this count cannot see.

use clippy_utils::diagnostics::span_lint_hir;
use rustc_data_structures::fx::FxIndexSet;
use rustc_hir::{Item, ItemKind};
use rustc_lint::{LateContext, LateLintPass, LintContext, declare_lint, declare_lint_pass};

declare_lint! {
    /// See the module documentation for the exemptions.
    pub(crate) AGENT_LONE_IMPL,
    Warn,
    "trait has exactly one implementing type in the crate"
}

declare_lint_pass!(LoneImpl => [AGENT_LONE_IMPL]);

impl<'tcx> LateLintPass<'tcx> for LoneImpl {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx Item<'tcx>) {
        let ItemKind::Trait { ident, items, .. } = item.kind else {
            return;
        };
        if items.is_empty() || item.span.from_expansion() {
            return;
        }
        let impls = cx.tcx.trait_impls_of(item.owner_id.to_def_id());
        if !impls.blanket_impls().is_empty() {
            return;
        }
        // Keys of `non_blanket_impls` group types by outer shape (`&A` and `&B`
        // share one), so distinct types come from each impl's own self type.
        let mut implementers = FxIndexSet::default();
        for &impl_id in impls.non_blanket_impls().values().flatten() {
            implementers.insert(
                cx.tcx
                    .type_of(impl_id)
                    .instantiate_identity()
                    .skip_norm_wip(),
            );
        }
        let (Some(sole_type), 1) = (implementers.first(), implementers.len()) else {
            return;
        };
        // A `#[cfg(test)]` double exists only in a test build, so only that
        // build can say none was found.
        let double_note = if cx.sess().is_test_crate() {
            " and no test double"
        } else {
            ""
        };
        span_lint_hir(
            cx,
            AGENT_LONE_IMPL,
            item.hir_id(),
            ident.span,
            format!(
                "trait `{ident}` has exactly one implementing type in this compilation (`{sole_type}`){double_note}; using `{sole_type}` directly removes the trait and the indirection at its uses"
            ),
        );
    }
}
