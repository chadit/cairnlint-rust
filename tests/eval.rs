//! End-to-end eval: the installed `cargo dylint` loads this library by path
//! and checks the fixture crate in `testdata/eval/` with the arguments dotai's lint.sh
//! passes. The findings must match the fixture's `// eval:` markers exactly.
//! Both tests are ignored because plain `cargo test` cannot assume an
//! installed cargo-dylint; `make eval` runs them, and CI runs `make test`.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use cargo_metadata::diagnostic::DiagnosticLevel;
use cargo_metadata::{CompilerMessage, Message, Target};

/// The fixture crate, relative to the repository root. dotai's lint.sh skips
/// any `testdata` path, so its must-flag code does not fail the owner's lint.
const FIXTURE: &str = "testdata/eval";

/// The fixture directories whose `.rs` files can hold markers.
const MARKED_DIRS: [&str; 2] = ["src", "tests"];

/// A marker comment names the finding on the line below it, because rustfmt
/// moves a trailing comment off a line that opens a block.
const MARKER: &str = "// eval: ";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Expect {
    Flag,
    Clean,
}

impl fmt::Display for Expect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Flag => "flag",
            Self::Clean => "clean",
        })
    }
}

/// `file` is relative to the fixture root, as cargo reports it.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Finding {
    file: String,
    line: usize,
    lint: String,
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{} {}", self.file, self.line, self.lint)
    }
}

struct Marker {
    expect: Expect,
    finding: Finding,
}

#[test]
#[ignore = "needs cargo-dylint; run `make eval`"]
fn every_lint_has_a_flagged_and_a_clean_case() {
    let lints = library_lints();
    let markers = fixture_markers();
    let missing_cases = lints
        .iter()
        .flat_map(|lint| [Expect::Flag, Expect::Clean].map(|expect| (lint, expect)))
        .filter(|&(lint, expect)| {
            !markers
                .iter()
                .any(|marker| marker.expect == expect && marker.finding.lint == *lint)
        })
        .map(|(lint, expect)| format!("{lint} has no `{MARKER}{expect} {lint}` case"));
    let unknown_lints = markers
        .iter()
        .filter(|marker| !lints.contains(&marker.finding.lint))
        .map(|marker| {
            format!(
                "{} names a lint the library does not register",
                marker.finding
            )
        });
    let gaps: Vec<String> = missing_cases.chain(unknown_lints).collect();
    assert!(
        gaps.is_empty(),
        "the eval fixture in {FIXTURE}/ is incomplete:\n{}",
        gaps.join("\n")
    );
}

#[test]
#[ignore = "needs cargo-dylint; run `make eval`"]
fn cargo_dylint_reports_exactly_the_flagged_lines() {
    let lints = library_lints();
    let expected: BTreeSet<Finding> = fixture_markers()
        .into_iter()
        .filter(|marker| marker.expect == Expect::Flag)
        .map(|marker| marker.finding)
        .collect();
    let reported = reported_findings(&lints);
    let lost: Vec<String> = expected
        .difference(&reported)
        .map(ToString::to_string)
        .collect();
    let new: Vec<String> = reported
        .difference(&expected)
        .map(ToString::to_string)
        .collect();
    assert!(
        lost.is_empty() && new.is_empty(),
        "lost findings, marked `flag` but not reported:\n{}\nnew findings, reported but not marked `flag`:\n{}",
        lost.join("\n"),
        new.join("\n")
    );
}

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// Runs `cargo dylint` from the repository root. A missing cargo-dylint fails
/// the test rather than skipping it, because a skipped eval reads as a pass.
fn cargo_dylint(args: &[&str]) -> Output {
    let output = Command::new("cargo")
        .arg("dylint")
        .args(args)
        .current_dir(repo_root())
        .output()
        .unwrap_or_else(|error| panic!("could not run cargo: {error}"));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("no such command: `dylint`"),
        "cargo-dylint is not installed; run `cargo install cargo-dylint dylint-link`"
    );
    output
}

/// Reads the lint names from the built library, so a lint added to
/// `register_lints` shows up here without anyone updating a list.
fn library_lints() -> BTreeSet<String> {
    let output = cargo_dylint(&["list", "--path", "."]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "cargo dylint list failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    // The library's name sits at the margin, and each of its lints follows on
    // an indented line of name, level, and description.
    let lints: BTreeSet<String> = stdout
        .lines()
        .filter(|line| line.starts_with(char::is_whitespace))
        .filter_map(|line| line.split_whitespace().next())
        .map(str::to_owned)
        .collect();
    assert!(
        !lints.is_empty(),
        "cargo dylint list printed no lints:\n{stdout}"
    );
    lints
}

fn reported_findings(lints: &BTreeSet<String>) -> BTreeSet<Finding> {
    let manifest = format!("{FIXTURE}/Cargo.toml");
    let output = cargo_dylint(&[
        "--path",
        ".",
        "--manifest-path",
        &manifest,
        "--",
        "--all-targets",
        "--all-features",
        "--message-format=json",
    ]);
    let mut reports = Reports::default();
    for message in Message::parse_stream(output.stdout.as_slice()) {
        reports.record(
            message.unwrap_or_else(|error| panic!("unreadable cargo output: {error}")),
            lints,
        );
    }
    assert!(
        output.status.success() && reports.errors.is_empty(),
        "cargo dylint could not build or check the fixture:\n{}\n{}",
        reports.errors.join("\n"),
        String::from_utf8_lossy(&output.stderr)
    );
    reports.kept()
}

/// A target's name and root file, which tell its builds apart from another
/// target's.
type TargetKey = (String, String);

/// A finding as lint.sh groups it: its target, its lint, and the file, line,
/// and column of every primary span.
type ReportKey = (TargetKey, String, Vec<(String, usize, usize)>);

/// `--all-targets` checks the fixture's lib and bin twice, plain and as a test
/// build, and only the test build sees a test double or a value only a test
/// passes. lint.sh keeps a finding only when every build of its target reports
/// it, so the eval counts builds and reports the same way.
#[derive(Default)]
struct Reports {
    builds: BTreeMap<TargetKey, usize>,
    reports: BTreeMap<ReportKey, usize>,
    errors: Vec<String>,
}

impl Reports {
    fn record(&mut self, message: Message, lints: &BTreeSet<String>) {
        match message {
            Message::CompilerArtifact(artifact) => {
                *self.builds.entry(target_key(&artifact.target)).or_default() += 1;
            }
            Message::CompilerMessage(compiler) => self.record_diagnostic(compiler, lints),
            _ => {}
        }
    }

    fn record_diagnostic(&mut self, compiler: CompilerMessage, lints: &BTreeSet<String>) {
        let target = target_key(&compiler.target);
        let diagnostic = compiler.message;
        if diagnostic.level == DiagnosticLevel::Error {
            self.errors
                .push(diagnostic.rendered.unwrap_or(diagnostic.message));
            return;
        }
        let Some(lint) = diagnostic
            .code
            .map(|code| code.code)
            .filter(|code| lints.contains(code))
        else {
            return;
        };
        let spans = diagnostic
            .spans
            .iter()
            .filter(|span| span.is_primary)
            .map(|span| (span.file_name.clone(), span.line_start, span.column_start))
            .collect();
        *self.reports.entry((target, lint, spans)).or_default() += 1;
    }

    /// A target with no artifact line still needs one report, as in lint.sh.
    fn kept(self) -> BTreeSet<Finding> {
        let builds = self.builds;
        self.reports
            .into_iter()
            .filter(|((target, ..), count)| *count >= builds.get(target).copied().unwrap_or(1))
            .flat_map(|((_, lint, spans), _)| {
                spans.into_iter().map(move |(file, line, _)| Finding {
                    file,
                    line,
                    lint: lint.clone(),
                })
            })
            .collect()
    }
}

fn target_key(target: &Target) -> TargetKey {
    (target.name.clone(), target.src_path.to_string())
}

fn fixture_markers() -> Vec<Marker> {
    let root = repo_root().join(FIXTURE);
    let mut markers = Vec::new();
    for dir in MARKED_DIRS {
        for path in rust_files(&root.join(dir)) {
            markers.extend(file_markers(&root, &path));
        }
    }
    markers
}

fn rust_files(dir: &Path) -> Vec<PathBuf> {
    let entries = fs::read_dir(dir).unwrap_or_else(|error| panic!("{}: {error}", dir.display()));
    entries
        .map(|entry| {
            entry
                .unwrap_or_else(|error| panic!("{}: {error}", dir.display()))
                .path()
        })
        .filter(|path| path.extension().is_some_and(|extension| extension == "rs"))
        .collect()
}

fn file_markers(root: &Path, path: &Path) -> Vec<Marker> {
    let file = path
        .strip_prefix(root)
        .expect("every marked file sits under the fixture root")
        .to_string_lossy()
        .into_owned();
    let text = fs::read_to_string(path).unwrap_or_else(|error| panic!("{file}: {error}"));
    text.lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let marker = line.trim_start().strip_prefix(MARKER)?;
            let Some((expect, lint)) = marker.trim().split_once(' ') else {
                panic!(
                    "{file}:{}: a marker needs `flag` or `clean` and a lint",
                    index + 1
                );
            };
            let expect = match expect {
                "flag" => Expect::Flag,
                "clean" => Expect::Clean,
                other => panic!("{file}:{}: unknown marker kind `{other}`", index + 1),
            };
            // `index` is the marker's zero-based line, so the line below it is
            // `index + 2` in cargo's one-based numbering.
            Some(Marker {
                expect,
                finding: Finding {
                    file: file.clone(),
                    line: index + 2,
                    lint: lint.to_owned(),
                },
            })
        })
        .collect()
}
