//! `agent_passthrough` flags an inherent method whose body is one call that
//! hands every parameter, unchanged and in order, to a method on one of
//! `self`'s fields. When a single-field struct's inherent methods all forward,
//! it reports once at the type instead: the newtype adds nothing callers could
//! not get by holding the wrapped value. Fowler calls that shape a Middle Man.
//!
//! Not reported: trait impl methods, because a trait's contract and the orphan
//! rule are the usual reasons a newtype forwards; associated functions without
//! `self`, such as constructors, which neither forward nor count toward the
//! type's method total; async methods; test code; macro output; and methods
//! taking `self: Box<Self>` or another explicit receiver. Those last ones still
//! count toward the method total, since any of them can do real work.
//!
//! Known problems: forwarding is sometimes the point, as in an adapter at a
//! module boundary or a newtype that validates in its constructor. The lint
//! reports the shape and leaves the reason to the reader.

use clippy_utils::diagnostics::span_lint_hir;
use clippy_utils::res::MaybeResPath;
use clippy_utils::source::snippet_opt;
use rustc_data_structures::fx::FxIndexMap;
use rustc_hir::{Body, Expr, ExprKind, HirId, ImplItem, ImplItemImplKind, ImplItemKind, Param};
use rustc_lint::{LateContext, LateLintPass, declare_lint, impl_lint_pass};
use rustc_span::def_id::DefId;
use rustc_span::{Span, Symbol};

use crate::block::sole_expr;
use crate::test_code::is_test_code;

declare_lint! {
    /// See the module documentation for the exemptions.
    pub(crate) AGENT_PASSTHROUGH,
    Warn,
    "method only forwards to a field's method, or a type whose every method forwards"
}

/// Methods are grouped per type across every inherent impl block before
/// anything is reported, because the type-level finding needs the whole set.
#[derive(Default)]
pub(crate) struct Passthrough {
    by_type: FxIndexMap<DefId, TypeMethods>,
}

impl_lint_pass!(Passthrough => [AGENT_PASSTHROUGH]);

/// The total sits beside the forwarders because it is what separates a lone
/// forwarder on a working type from a type whose whole method set forwards.
#[derive(Default)]
struct TypeMethods {
    forwarders: Vec<Forwarder>,
    total: usize,
}

struct Forwarder {
    hir_id: HirId,
    span: Span,
    name: Symbol,
    target: String,
}

impl<'tcx> LateLintPass<'tcx> for Passthrough {
    fn check_impl_item(&mut self, cx: &LateContext<'tcx>, impl_item: &'tcx ImplItem<'tcx>) {
        let ImplItemKind::Fn(sig, body_id) = impl_item.kind else {
            return;
        };
        if !matches!(impl_item.impl_kind, ImplItemImplKind::Inherent { .. })
            || !cx.tcx.associated_item(impl_item.owner_id).is_method()
        {
            return;
        }
        let impl_id = cx.tcx.hir_get_parent_item(impl_item.hir_id());
        let Some(adt) = cx
            .tcx
            .type_of(impl_id)
            .instantiate_identity()
            .skip_norm_wip()
            .ty_adt_def()
        else {
            return;
        };
        let methods = self.by_type.entry(adt.did()).or_default();
        methods.total += 1;
        // These methods still count toward the total, because one of them can do
        // the real work that makes "every method forwards" false.
        if !sig.decl.implicit_self().has_implicit_self()
            || impl_item.span.from_expansion()
            || is_test_code(cx, impl_item.hir_id())
        {
            return;
        }
        let Some(target) = forward_target(cx, cx.tcx.hir_body(body_id)) else {
            return;
        };
        let forwarder = Forwarder {
            hir_id: impl_item.hir_id(),
            span: impl_item.ident.span,
            name: impl_item.ident.name,
            target,
        };
        // An allowed or expected forwarder must not feed the type-level finding,
        // which lands on the struct where the method's attribute cannot reach.
        // Reporting it here keeps an `#[expect]` fulfilled; `#[allow]` drops it.
        let level = cx
            .tcx
            .lint_level_spec_at_node(AGENT_PASSTHROUGH, forwarder.hir_id);
        if level.is_allow() || level.is_expect() {
            report_method(cx, adt.did(), &forwarder);
        } else {
            methods.forwarders.push(forwarder);
        }
    }

    /// One finding at the type keeps a wrapper with eight methods from
    /// producing eight diagnostics that all say the same thing.
    fn check_crate_post(&mut self, cx: &LateContext<'tcx>) {
        for (&type_id, methods) in &self.by_type {
            if methods.forwarders.is_empty() {
                continue;
            }
            if methods.forwarders.len() == methods.total
                && report_wrapper(cx, type_id, methods.total)
            {
                continue;
            }
            for method in &methods.forwarders {
                report_method(cx, type_id, method);
            }
        }
    }
}

fn report_method(cx: &LateContext<'_>, type_id: DefId, method: &Forwarder) {
    let type_name = cx.tcx.item_name(type_id);
    span_lint_hir(
        cx,
        AGENT_PASSTHROUGH,
        method.hir_id,
        method.span,
        format!(
            "method `{type_name}::{}` only forwards its arguments to `{}` and adds no behavior of its own",
            method.name, method.target
        ),
    );
}

/// Reports the type-level finding and returns true when the type is a local
/// struct with exactly one field. A wider struct whose methods all forward is
/// coordinating several values, which is a different shape. Only a struct gets
/// here, since an enum method cannot reach a field through `self`, so
/// `non_enum_variant` never sees an enum.
fn report_wrapper(cx: &LateContext<'_>, type_id: DefId, total: usize) -> bool {
    let Some(local_id) = type_id.as_local() else {
        return false;
    };
    let adt = cx.tcx.adt_def(type_id);
    if adt.non_enum_variant().fields.len() != 1 {
        return false;
    }
    let Some(ident_span) = cx.tcx.def_ident_span(type_id) else {
        return false;
    };
    let type_name = cx.tcx.item_name(type_id);
    let which_methods = if total == 1 {
        String::from("its only method forwards")
    } else {
        format!("every one of its {total} methods forwards")
    };
    span_lint_hir(
        cx,
        AGENT_PASSTHROUGH,
        cx.tcx.local_def_id_to_hir_id(local_id),
        ident_span,
        format!(
            "type `{type_name}` wraps one field and {which_methods}; callers can hold the wrapped value directly"
        ),
    );
    true
}

/// Returns the rendered call target when the body is a single method call on a
/// field of `self` that passes every other parameter through unchanged. A body
/// that branches, reshapes an argument, or handles an error is doing work.
fn forward_target(cx: &LateContext<'_>, body: &Body<'_>) -> Option<String> {
    let [self_param, params @ ..] = body.params else {
        return None;
    };
    let ExprKind::Block(block, _) = body.value.kind else {
        return None;
    };
    let ExprKind::MethodCall(segment, receiver, args, _) = sole_expr(block)?.kind else {
        return None;
    };
    if !is_field_of(receiver, self_param.pat.hir_id) || !passes_params_through(args, params) {
        return None;
    }
    let mut target = snippet_opt(cx, receiver.span)?;
    target.push('.');
    target.push_str(segment.ident.as_str());
    Some(target)
}

/// True for `self.a` or `self.a.b`, a chain of at least one field access that
/// starts at the `self` parameter.
fn is_field_of(receiver: &Expr<'_>, self_id: HirId) -> bool {
    let ExprKind::Field(mut base, _) = receiver.kind else {
        return false;
    };
    while let ExprKind::Field(inner, _) = base.kind {
        base = inner;
    }
    base.res_local_id() == Some(self_id)
}

fn passes_params_through(args: &[Expr<'_>], params: &[Param<'_>]) -> bool {
    args.len() == params.len()
        && args
            .iter()
            .zip(params)
            .all(|(arg, param)| arg.res_local_id() == Some(param.pat.hir_id))
}
