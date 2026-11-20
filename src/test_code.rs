use std::path::Path;

use clippy_utils::is_in_test;
use rustc_hir::HirId;
use rustc_lint::{LateContext, LintContext};

/// Test code is `#[cfg(test)]` and `#[test]` code, plus every item of an
/// integration test crate. Such a crate marks only its test functions, so
/// `is_in_test` alone would lint the helpers beside them.
pub(crate) fn is_test_code(cx: &LateContext<'_>, hir_id: HirId) -> bool {
    is_in_test(cx.tcx, hir_id) || is_integration_test_crate(cx)
}

/// Cargo builds each `tests/*.rs` file, and each `tests/<name>/main.rs`, as its
/// own `--test` crate rooted at that file, so the root's path identifies one.
fn is_integration_test_crate(cx: &LateContext<'_>) -> bool {
    if !cx.sess().is_test_crate() {
        return false;
    }
    let Some(root) = cx.sess().local_crate_source_file() else {
        return false;
    };
    root.local_path().is_some_and(is_integration_test_root)
}

fn is_integration_test_root(root: &Path) -> bool {
    let named_tests = |directory: Option<&Path>| {
        directory
            .and_then(Path::file_name)
            .is_some_and(|name| name == "tests")
    };
    let directory = root.parent();
    named_tests(directory)
        || (root.file_name().is_some_and(|name| name == "main.rs")
            && named_tests(directory.and_then(Path::parent)))
}
