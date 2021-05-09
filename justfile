log := 'warn'
bt := '0'

export RUST_LOG := log

export RUST_BACKTRACE := bt

watch *args='check --all --tests':
	cargo watch --clear --exec '{{args}}'


test *args:
	cargo test --all -- {{args}}

integration *args:
	cargo test --all -- --test-threads 1 --ignored {{args}}

run:
	cargo run

env:
	env

lint:
	./bin/lint

fmt:
	cargo +nightly fmt --all

fmt-check:
	cargo +nightly fmt --all -- --check

check:
	cargo check --all --tests

deps:
	cargo install sqlx-cli

push remote: try
	git diff --no-ext-diff --quiet --exit-code
	git branch | grep '* master'
	git push {{remote}}

try: fmt-check check clippy test lint integration

pr remote: (push remote)
	hub pull-request -o

clippy:
	cargo clippy --all -- \
		-D clippy::all \
		-D clippy::pedantic \
		-D clippy::restriction \
		-A clippy::blanket-clippy-restriction-lints \
		-A clippy::enum-glob-use \
		-A clippy::expect-used \
		-A clippy::if-not-else \
		-A clippy::implicit-return \
		-A clippy::indexing-slicing \
		-A clippy::integer-arithmetic \
		-A clippy::missing-docs-in-private-items \
		-A clippy::missing-errors-doc \
		-A clippy::missing-inline-in-public-items \
		-A clippy::must-use-candidate \
		-A clippy::non-ascii-literal \
		-A clippy::option-if-let-else \
		-A clippy::panic \
		-A clippy::pattern-type-mismatch \
		-A clippy::print-stdout \
		-A clippy::print-stderr \
		-A clippy::shadow-reuse \
		-A clippy::todo \
		-A clippy::trivially-copy-pass-by-ref \
		-A clippy::unseparated-literal-suffix \
		-A clippy::wildcard-enum-match-arm \
		-A clippy::wildcard-imports
