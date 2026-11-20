use rustc_hir::{Block, Expr, ExprKind, StmtKind};

/// Returns the one expression a block consists of: its tail expression or its
/// single expression statement, with a leading `return` peeled off. A block
/// with any other statement, or more than one, is doing more than one thing.
pub(crate) fn sole_expr<'tcx>(block: &Block<'tcx>) -> Option<&'tcx Expr<'tcx>> {
    let expr = match (block.stmts, block.expr) {
        ([], Some(tail)) => tail,
        ([stmt], None) => match stmt.kind {
            StmtKind::Semi(expr) | StmtKind::Expr(expr) => expr,
            _ => return None,
        },
        _ => return None,
    };
    match expr.kind {
        ExprKind::Ret(Some(value)) => Some(value),
        _ => Some(expr),
    }
}
