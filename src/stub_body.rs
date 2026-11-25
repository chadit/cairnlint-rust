//! `agent_stub_body` flags a function whose body does nothing its name or doc
//! comment claims. It reports five shapes:
//!
//! - a body that is only `todo!()` or `unimplemented!()`, with or without a
//!   message, because Clippy's `todo` and `unimplemented` lints sit in the
//!   restriction group, which is off by default;
//! - an empty body under a doc comment that describes behavior;
//! - a lone `Err(..)` whose text or variant reads like an unfinished stub, such
//!   as `Err("not implemented".into())` or `Err(Error::NotImplemented)`;
//! - a lone canned `Result` (`Ok(())`, `Ok(0)`, `Err("bad".into())`) from a
//!   function that takes parameters or carries a doc comment;
//! - a lone canned value (`None`, `0`, `Default::default()`, `Vec::new()`)
//!   under a doc comment.
//!
//! A body that panics any other way, such as `panic!` or `unreachable!`, is
//! left to Clippy, because that panic often marks a real invariant rather than
//! unfinished work.
//!
//! Never reported: test code, macro output, and required trait methods (no
//! body). The other shapes are also not reported for an empty body whose doc
//! calls it a no-op, a trait default method whose doc does the same, an
//! undocumented method without parameters (a null object such as a `Drop` or
//! `flush` that has nothing to do), or a constructor that returns
//! `Self::default()`. A `todo!()` or `unimplemented!()` body gets none of these
//! exemptions, because calling it panics whatever its doc says.
//!
//! Known problems: a sink or null-object method that takes parameters, such as
//! `fmt::Write::write_str` on a type that discards output, is reported.

use std::ops::ControlFlow;

use clippy_utils::diagnostics::span_lint_hir;
use clippy_utils::macros::root_macro_call;
use clippy_utils::res::{MaybeDef, MaybeQPath, MaybeResPath};
use clippy_utils::source::snippet_opt;
use clippy_utils::visitors::for_each_expr;
use clippy_utils::{fn_def_id, get_async_closure_expr, is_default_equivalent};
use rustc_ast::attr::AttributeExt;
use rustc_attr_ir::lang_items::LangItem;
use rustc_hir::def::DefKind;
use rustc_hir::intravisit::FnKind;
use rustc_hir::{Block, Body, Expr, ExprKind, FnDecl, HirId, Node};
use rustc_lint::{LateContext, LateLintPass, declare_lint, declare_lint_pass};
use rustc_middle::ty::Ty;
use rustc_span::def_id::LocalDefId;
use rustc_span::{Span, Symbol, sym};

use crate::block::sole_expr;
use crate::test_code::is_test_code;

declare_lint! {
    /// Flags a function whose body does nothing its name or doc comment
    /// claims: a `todo!()` or `unimplemented!()` body, an empty body under a
    /// doc, a "not implemented" error, or a lone canned return. See the module
    /// documentation for the exemptions.
    pub(crate) AGENT_STUB_BODY,
    Warn,
    "function body does nothing its name or doc comment claims"
}

declare_lint_pass!(StubBody => [AGENT_STUB_BODY]);

/// The two facts about a signature that decide whether a canned body is
/// suspicious. Inputs it never reads sharpen the case, and so does a doc
/// comment that promises behavior.
struct Claims {
    has_doc: bool,
    has_params: bool,
}

#[derive(Clone, Copy)]
enum BodyShape<'tcx> {
    Empty,
    SoleReturn(&'tcx Expr<'tcx>),
    Working,
}

impl<'tcx> LateLintPass<'tcx> for StubBody {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        kind: FnKind<'tcx>,
        decl: &'tcx FnDecl<'tcx>,
        body: &'tcx Body<'tcx>,
        span: Span,
        def_id: LocalDefId,
    ) {
        if matches!(kind, FnKind::Closure) || span.from_expansion() {
            return;
        }
        let hir_id = cx.tcx.local_def_id_to_hir_id(def_id);
        if is_test_code(cx, hir_id) {
            return;
        }
        let Some(block) = fn_block(cx, body) else {
            return;
        };
        let name = cx.tcx.item_name(def_id.to_def_id());
        let shape = body_shape(block);
        let message = placeholder_message(cx, shape, name).or_else(|| {
            let doc = doc_text(cx, hir_id);
            if is_noop_trait_default(cx, def_id, &doc) {
                return None;
            }
            let claims = Claims {
                has_doc: !doc.trim().is_empty(),
                has_params: decl.inputs.len()
                    > usize::from(decl.implicit_self().has_implicit_self()),
            };
            match shape {
                BodyShape::Empty if claims.has_doc && !mentions_noop(&doc) => Some(format!(
                    "`{name}` has an empty body but its doc describes behavior: implement it or drop the claim"
                )),
                BodyShape::SoleReturn(returned) => {
                    sole_return_message(cx, def_id, returned, &claims, name)
                }
                BodyShape::Empty | BodyShape::Working => None,
            }
        });
        if let Some(message) = message {
            let ident_span = cx.tcx.def_ident_span(def_id).unwrap_or(span);
            span_lint_hir(cx, AGENT_STUB_BODY, hir_id, ident_span, message);
        }
    }
}

fn placeholder_message(cx: &LateContext<'_>, shape: BodyShape<'_>, name: Symbol) -> Option<String> {
    let BodyShape::SoleReturn(returned) = shape else {
        return None;
    };
    let call = root_macro_call(returned.span)?;
    if !matches!(
        cx.tcx.get_diagnostic_name(call.def_id),
        Some(sym::todo_macro | sym::unimplemented_macro)
    ) {
        return None;
    }
    // A call spread over several lines would split the warning's headline, so
    // it shows as the bare macro instead.
    let text = snippet_opt(cx, call.span)
        .filter(|text| !text.contains('\n'))
        .unwrap_or_else(|| format!("{}!()", cx.tcx.item_name(call.def_id)));
    Some(format!(
        "the body of `{name}` is only `{text}`: wire up a real implementation before use"
    ))
}

fn sole_return_message<'tcx>(
    cx: &LateContext<'tcx>,
    def_id: LocalDefId,
    returned: &'tcx Expr<'tcx>,
    claims: &Claims,
    name: rustc_span::Symbol,
) -> Option<String> {
    if let Some(text) = stub_sentinel(cx, returned) {
        return Some(format!(
            "`{name}` returns a stub (`{text}`): wire up a real implementation before use"
        ));
    }
    if !is_canned(cx, returned) {
        return None;
    }
    let returned_ty = cx.typeck_results().expr_ty(returned);
    if returns_own_type(cx, def_id, returned_ty) {
        return None;
    }
    let returns_result = returned_ty.is_diag_item(cx, sym::Result);
    if returns_result && (claims.has_params || claims.has_doc) {
        return Some(format!(
            "`{name}` returns without doing the work its signature implies: a one-line function that can fail but never acts is usually an unfinished stub"
        ));
    }
    if claims.has_doc && !returns_result {
        return Some(format!(
            "`{name}` returns a canned value; verify it does what its doc claims rather than standing in as a stub"
        ));
    }
    None
}

fn doc_text(cx: &LateContext<'_>, hir_id: HirId) -> String {
    let mut doc = String::new();
    for (line, ..) in cx
        .tcx
        .hir_attrs(hir_id)
        .iter()
        .filter_map(AttributeExt::doc_str_and_fragment_kind)
    {
        if !doc.is_empty() {
            doc.push('\n');
        }
        doc.push_str(line.as_str());
    }
    doc
}

/// A trait default whose doc says it does nothing is a hook implementors
/// override, so its empty or canned body is the documented behavior.
fn is_noop_trait_default(cx: &LateContext<'_>, def_id: LocalDefId, doc: &str) -> bool {
    matches!(cx.tcx.hir_node_by_def_id(def_id), Node::TraitItem(_)) && mentions_noop(doc)
}

/// An `async fn` lowers to a coroutine closure, so its written block sits one
/// body further in.
fn fn_block<'tcx>(cx: &LateContext<'tcx>, body: &'tcx Body<'tcx>) -> Option<&'tcx Block<'tcx>> {
    let value = get_async_closure_expr(cx.tcx, body.value).unwrap_or(body.value);
    match value.kind {
        ExprKind::Block(block, _) => Some(block),
        _ => None,
    }
}

/// A bare `return;` hands back `()`, which is the same as an empty body.
fn body_shape<'tcx>(block: &'tcx Block<'tcx>) -> BodyShape<'tcx> {
    if block.stmts.is_empty() && block.expr.is_none() {
        return BodyShape::Empty;
    }
    match sole_expr(block) {
        Some(Expr {
            kind: ExprKind::Ret(None),
            ..
        }) => BodyShape::Empty,
        Some(returned) => BodyShape::SoleReturn(returned),
        None => BodyShape::Working,
    }
}

/// A canned value is fixed at compile time: a literal, a unit or tuple of
/// canned values, an `Ok` or `Some` around one, an `Err` around an error built
/// without any runtime input, or anything equal to `Default::default()`.
fn is_canned<'tcx>(cx: &LateContext<'tcx>, expr: &'tcx Expr<'tcx>) -> bool {
    match expr.kind {
        ExprKind::Lit(_) => true,
        ExprKind::Tup(items) => items.iter().all(|item| is_canned(cx, item)),
        ExprKind::Call(callee, [arg])
            if is_ctor(cx, callee, LangItem::OptionSome)
                || is_ctor(cx, callee, LangItem::ResultOk) =>
        {
            is_canned(cx, arg)
        }
        ExprKind::Call(callee, [arg]) if is_ctor(cx, callee, LangItem::ResultErr) => {
            is_canned_error(cx, arg)
        }
        _ => is_default_equivalent(cx, expr),
    }
}

fn is_ctor(cx: &LateContext<'_>, callee: &Expr<'_>, item: LangItem) -> bool {
    callee.res(cx).ctor_parent(cx).is_lang_item(cx, item)
}

/// Locals and calls that a macro writes inside its own expansion, such as the
/// binding `anyhow!` builds its error in, are not runtime input from the caller.
fn is_canned_error<'tcx>(cx: &LateContext<'tcx>, error: &'tcx Expr<'tcx>) -> bool {
    for_each_expr(cx.tcx, error, |expr| {
        let reads_runtime = expr.res_local_id().is_some() || is_delegating_call(cx, expr);
        if reads_runtime && !expr.span.from_expansion() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    })
    .is_none()
}

/// Calls that only build a value from their arguments, so an error made of
/// them is canned when the arguments are. `other` is `io::Error::other`.
const BUILDING_CALLS: [&str; 6] = ["new", "from", "into", "to_string", "to_owned", "other"];

/// A call that reads state, such as `io::Error::last_os_error()`, does real
/// work. Constructors and the plain conversions in `BUILDING_CALLS` do not;
/// the walk in `is_canned_error` still checks their arguments.
fn is_delegating_call(cx: &LateContext<'_>, expr: &Expr<'_>) -> bool {
    if !matches!(expr.kind, ExprKind::Call(..) | ExprKind::MethodCall(..)) {
        return false;
    }
    let Some(callee) = fn_def_id(cx, expr) else {
        return true;
    };
    !matches!(cx.tcx.def_kind(callee), DefKind::Ctor(..))
        && !BUILDING_CALLS.contains(&cx.tcx.item_name(callee).as_str())
}

/// Returns the source text of a canned `Err` whose message or variant name
/// reads like an unfinished stub.
fn stub_sentinel<'tcx>(cx: &LateContext<'tcx>, returned: &'tcx Expr<'tcx>) -> Option<String> {
    let ExprKind::Call(callee, [error]) = returned.kind else {
        return None;
    };
    if !is_ctor(cx, callee, LangItem::ResultErr) || !is_canned_error(cx, error) {
        return None;
    }
    let text = snippet_opt(cx, error.span.source_callsite())?;
    mentions_stub(&text).then_some(text)
}

/// `Self::default()` in a constructor is the idiom for an empty value, not a
/// stub standing in for one.
fn returns_own_type<'tcx>(
    cx: &LateContext<'tcx>,
    def_id: LocalDefId,
    returned_ty: Ty<'tcx>,
) -> bool {
    let parent = cx.tcx.local_parent(def_id);
    if !matches!(
        cx.tcx.def_kind(parent),
        rustc_hir::def::DefKind::Impl { .. }
    ) {
        return false;
    }
    cx.tcx
        .type_of(parent)
        .instantiate_identity()
        .skip_norm_wip()
        == returned_ty
}

/// Splits text into lowercase words at punctuation, whitespace, and camelCase
/// boundaries, so `NotImplemented`, `not_implemented`, and "not implemented"
/// all read as the same two words.
fn words(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut current = String::new();
    let mut previous_lower = false;
    for character in text.chars() {
        let boundary = !character.is_alphanumeric() || (character.is_uppercase() && previous_lower);
        if boundary && !current.is_empty() {
            found.push(std::mem::take(&mut current));
        }
        if character.is_alphanumeric() {
            current.extend(character.to_lowercase());
        }
        previous_lower = character.is_lowercase();
    }
    if !current.is_empty() {
        found.push(current);
    }
    found
}

fn has_word_or_pair(words: &[String], singles: &[&str], pairs: &[(&str, &str)]) -> bool {
    words.iter().any(|word| singles.contains(&word.as_str()))
        || words
            .windows(2)
            .any(|pair| pairs.contains(&(pair[0].as_str(), pair[1].as_str())))
}

/// Kept broad on purpose: recall matters more than precision here, because an
/// LLM dismisses the rare false hit.
fn mentions_stub(text: &str) -> bool {
    has_word_or_pair(
        &words(text),
        &[
            "unimplemented",
            "notimplemented",
            "todo",
            "stub",
            "placeholder",
            "fixme",
        ],
        &[("not", "implemented"), ("to", "do")],
    )
}

fn mentions_noop(doc: &str) -> bool {
    has_word_or_pair(
        &words(doc),
        &["noop"],
        &[("no", "op"), ("does", "nothing"), ("do", "nothing")],
    )
}
