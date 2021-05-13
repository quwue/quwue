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

ci: build test-all forbid fmt-check clippy clean-check

build:
	cargo build --all-features --all-targets

check:
	cargo check --all-features --all-targets

test-all:
	cargo test --all-features --all-targets

forbid:
	./bin/forbid

clean-check:
	git diff --no-ext-diff --quiet --exit-code

fmt:
	cargo +nightly fmt --all

fmt-check:
	cargo +nightly fmt --all -- --check

clippy:
	./bin/clippy

run:
	cargo run

env:
	env

deps:
	cargo install sqlx-cli

dev-deps:
	brew install gnuplot

push remote: ci
	! git branch | grep '* master'
	git push {{remote}}

pr remote: (push remote)
	hub pull-request -o

done remote branch=`git rev-parse --abbrev-ref HEAD`:
	git checkout master
	git diff --no-ext-diff --quiet --exit-code
	git pull --rebase {{remote}} master
	git diff --no-ext-diff --quiet --exit-code {{branch}}
	git branch -D {{branch}}
