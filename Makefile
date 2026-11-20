.PHONY: update-deps

# cargo dylint upgrade runs first so the Dylint pins resolve against the new nightly.
# pinact runs as a one-shot so nothing is installed; the token keeps it under the API rate limit.
update-deps: ## Move the nightly and clippy_utils rev, the Dylint pins, Cargo.lock, and action pins to the newest release
	cargo dylint upgrade && cargo upgrade --pinned --incompatible -p dylint_linting -p dylint_testing && cargo update
	GITHUB_TOKEN="$$(gh auth token)" go run github.com/suzuki-shunsuke/pinact/v3/cmd/pinact@latest run --update
