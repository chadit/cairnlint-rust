//! `agent_lone_param` flags a parameter that receives the same constant at
//! every call site in the crate. A parameter with one value is a choice nothing
//! makes: the branch it selects never varies, so the parameter, the argument at
//! each call site, and the unreachable half of the body come out together.
//!
//! A constant is a value rustc can fold at compile time (a literal, a `const`,
//! arithmetic over them) or a unit enum variant such as `None` or
//! `Mode::Strict`. Two different spellings of the same value count as one.
//!
//! Only free functions and inherent associated functions are counted, and only
//! when no other crate can reach them and they have two or more call sites. Not
//! reported: a function whose name is used as a value (`iter.map(helper)`),
//! since its other calls are invisible; trait methods and trait impl methods,
//! which dynamic or generic dispatch can reach; `self`; a parameter named `_`
//! or with a leading underscore; a function exported by symbol through
//! `#[no_mangle]` or `#[export_name]`; test code; and macro output. Call sites
//! in test code still count when tests are compiled.

use clippy_utils::consts::{ConstEvalCtxt, Constant};
use clippy_utils::diagnostics::span_lint_hir;
use clippy_utils::get_parent_expr;
use clippy_utils::res::MaybeQPath;
use clippy_utils::source::snippet_opt;
use rustc_data_structures::fx::{FxHashSet, FxIndexMap};
use rustc_hir::def::{CtorKind, DefKind, Res};
use rustc_hir::{Expr, ExprKind, Param, PatKind};
use rustc_lint::{LateContext, LateLintPass, declare_lint, impl_lint_pass};
use rustc_span::def_id::{DefId, LocalDefId};

use crate::test_code::is_test_code;

/// A single call site proves nothing about variation, so two are the minimum
/// before "always receives" means anything.
const MIN_CALL_SITES: usize = 2;

declare_lint! {
    /// Flags a parameter that receives the same constant at every call site in
    /// the crate, for functions with two or more call sites. See the module
    /// documentation for the exemptions.
    pub(crate) AGENT_LONE_PARAM,
    Warn,
    "parameter receives the same constant at every call site in the crate"
}

/// Call sites are spread across every body in the crate, so arguments are
/// folded as each call is visited and judged once the whole crate is seen.
#[derive(Default)]
pub(crate) struct LoneParam {
    calls: FxIndexMap<LocalDefId, CallSites>,
    used_as_value: FxHashSet<LocalDefId>,
}

impl_lint_pass!(LoneParam => [AGENT_LONE_PARAM]);

/// `arguments` lines up with the callee's full parameter list, `self`
/// included, so a method call and its `Type::method(receiver, ..)` spelling
/// fold into the same slots.
#[derive(Default)]
struct CallSites {
    count: usize,
    arguments: Vec<Argument>,
}

#[derive(PartialEq)]
enum ConstKey {
    Value(Constant),
    Variant(DefId),
}

enum Argument {
    Fixed { key: ConstKey, text: String },
    Varies,
}

impl<'tcx> LateLintPass<'tcx> for LoneParam {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx Expr<'tcx>) {
        match expr.kind {
            ExprKind::Call(callee, args) => {
                if let Some(callee_id) = local_fn(callee.res(cx)) {
                    self.record_call(cx, callee_id, args.iter());
                }
            }
            ExprKind::MethodCall(_, receiver, args, _) => {
                let method_def = cx.typeck_results().type_dependent_def(expr.hir_id);
                if let Some(callee_id) =
                    method_def.and_then(|(kind, id)| local_fn(Res::Def(kind, id)))
                {
                    self.record_call(cx, callee_id, std::iter::once(receiver).chain(args));
                }
            }
            ExprKind::Path(_) => {
                if let Some(fn_id) = local_fn(expr.res(cx))
                    && !is_callee(cx, expr)
                {
                    self.used_as_value.insert(fn_id);
                }
            }
            _ => {}
        }
    }

    fn check_crate_post(&mut self, cx: &LateContext<'tcx>) {
        for (&fn_id, sites) in &self.calls {
            if sites.count < MIN_CALL_SITES
                || self.used_as_value.contains(&fn_id)
                || !is_countable(cx, fn_id)
            {
                continue;
            }
            // A foreign `extern` function resolves like any other `fn` but has no body.
            let Some(body) = cx.tcx.hir_maybe_body_owned_by(fn_id) else {
                continue;
            };
            for (param, argument) in body.params.iter().zip(&sites.arguments) {
                report_fixed(cx, fn_id, param, argument, sites.count);
            }
        }
    }
}

impl LoneParam {
    fn record_call<'tcx>(
        &mut self,
        cx: &LateContext<'tcx>,
        callee_id: LocalDefId,
        args: impl Iterator<Item = &'tcx Expr<'tcx>>,
    ) {
        let sites = self.calls.entry(callee_id).or_default();
        sites.count += 1;
        if sites.count == 1 {
            sites.arguments = args.map(|arg| first_argument(cx, arg)).collect();
            return;
        }
        // Later calls only compare keys, so they skip the snippet the message
        // needs and the evaluation of a slot that already varies.
        for (slot, arg) in sites.arguments.iter_mut().zip(args) {
            if matches!(slot, Argument::Fixed { .. }) {
                merge(slot, const_key(cx, arg).as_ref());
            }
        }
    }
}

fn merge(slot: &mut Argument, seen: Option<&ConstKey>) {
    let same = matches!((&*slot, seen), (Argument::Fixed { key, .. }, Some(new)) if key == new);
    if !same {
        *slot = Argument::Varies;
    }
}

fn first_argument(cx: &LateContext<'_>, arg: &Expr<'_>) -> Argument {
    match (const_key(cx, arg), snippet_opt(cx, arg.span)) {
        (Some(key), Some(text)) => Argument::Fixed { key, text },
        _ => Argument::Varies,
    }
}

fn const_key(cx: &LateContext<'_>, arg: &Expr<'_>) -> Option<ConstKey> {
    match ConstEvalCtxt::new(cx).eval(arg) {
        Some(value) => Some(ConstKey::Value(value)),
        None => unit_variant(arg.res(cx)).map(ConstKey::Variant),
    }
}

fn unit_variant(resolution: Res) -> Option<DefId> {
    match resolution {
        Res::Def(DefKind::Ctor(_, CtorKind::Const), id) => Some(id),
        _ => None,
    }
}

fn local_fn(resolution: Res) -> Option<LocalDefId> {
    match resolution {
        Res::Def(DefKind::Fn | DefKind::AssocFn, id) => id.as_local(),
        _ => None,
    }
}

fn is_callee(cx: &LateContext<'_>, path: &Expr<'_>) -> bool {
    get_parent_expr(cx, path).is_some_and(
        |parent| matches!(parent.kind, ExprKind::Call(callee, _) if callee.hir_id == path.hir_id),
    )
}

/// Only a function no other crate or dispatch path can call has its full set
/// of call sites in view.
fn is_countable(cx: &LateContext<'_>, fn_id: LocalDefId) -> bool {
    let is_free_or_inherent = match cx.tcx.def_kind(fn_id) {
        DefKind::Fn => true,
        // A call to a trait impl method resolves to the trait's own method, so
        // an impl parent here is always an inherent impl.
        DefKind::AssocFn => matches!(
            cx.tcx.def_kind(cx.tcx.local_parent(fn_id)),
            DefKind::Impl { .. }
        ),
        _ => false,
    };
    let hir_id = cx.tcx.local_def_id_to_hir_id(fn_id);
    is_free_or_inherent
        && !cx.effective_visibilities.is_exported(fn_id)
        && !cx.tcx.codegen_fn_attrs(fn_id).contains_extern_indicator()
        && !cx.tcx.def_span(fn_id).from_expansion()
        && !is_test_code(cx, hir_id)
}

fn report_fixed(
    cx: &LateContext<'_>,
    fn_id: LocalDefId,
    param: &Param<'_>,
    argument: &Argument,
    count: usize,
) {
    let Argument::Fixed { text, .. } = argument else {
        return;
    };
    let PatKind::Binding(_, _, ident, None) = param.pat.kind else {
        return;
    };
    if ident.name == rustc_span::symbol::kw::SelfLower || ident.as_str().starts_with('_') {
        return;
    }
    let fn_name = cx.tcx.item_name(fn_id.to_def_id());
    span_lint_hir(
        cx,
        AGENT_LONE_PARAM,
        param.hir_id,
        param.pat.span,
        format!(
            "parameter `{ident}` of `{fn_name}` receives `{text}` at all {count} call sites in this crate; the choice it encodes is never made"
        ),
    );
}
