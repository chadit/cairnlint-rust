.PHONY: update-deps

# A hijacked release is usually found and pulled within days, so nothing younger than this many days enters the tree.
MIN_AGE_DAYS := 3

# cargo dylint upgrade runs first so the Dylint pins resolve against the new nightly.
# The cargo setting covers the whole line because cargo upgrade also rewrites Cargo.lock.
# pinact runs as a one-shot so nothing is installed; the token keeps it under the API rate limit.
update-deps: export CARGO_REGISTRY_GLOBAL_MIN_PUBLISH_AGE := $(MIN_AGE_DAYS) days
update-deps: ## Move the nightly and clippy_utils rev and the Dylint pins to the newest release, and Cargo.lock and action pins to the newest release at least MIN_AGE_DAYS old
	cargo dylint upgrade && cargo upgrade --pinned --incompatible -p dylint_linting -p dylint_testing && cargo update
	GITHUB_TOKEN="$$(gh auth token)" go run github.com/suzuki-shunsuke/pinact/v3/cmd/pinact@latest run --update --min-age $(MIN_AGE_DAYS)

.PHONY: test eval clean

# compiletest writes its build files under TMPDIR with fixed names, so cargo
# test runs with a private TMPDIR that the trap removes, pass or fail. A direct
# cargo test keeps them, which is how a .stderr file gets re-recorded.
test: ## Run cargo test in a private TMPDIR it then removes, then the eval, as CI does
	tmpdir=$$(mktemp -d) && trap 'rm -rf "$$tmpdir"' EXIT && TMPDIR="$$tmpdir" cargo test
	$(MAKE) --no-print-directory eval

# The eval's tests are ignored by plain cargo test because they need an
# installed cargo-dylint, which a plain cargo test cannot assume.
eval: ## Check the testdata/eval/ fixture crate through the installed cargo dylint and compare against its markers
	cargo test --test eval -- --ignored

# The eval fixture is its own workspace, so cargo clean at the root leaves its target/ behind.
clean: ## Remove target/ and the eval fixture's testdata/eval/target/
	cargo clean
	cargo clean --manifest-path testdata/eval/Cargo.toml
