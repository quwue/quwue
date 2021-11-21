set dotenv-load

bt := '0'
log := 'warn'

export EDITOR := 'vim'
export RUST_BACKTRACE := bt
export RUST_LOG := log

watch *args='lcheck --workspace --tests':
	cargo watch --clear --exec '{{args}}'

test *args:
	cargo test --workspace -- {{args}}

integration *args:
	cargo test --workspace -- --test-threads 1 --ignored {{args}}

ci: build test-all forbid fmt-check clippy clean-check

build:
	cargo build --workspace --all-features --all-targets

check:
	cargo check --workspace --all-features --all-targets

test-all:
	cargo test --workspace --all-features --all-targets

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
	cargo run -- --db-name quwue

env:
	env

# install system development dependencies with homebrew
install-dev-deps-homebrew:
  brew tap rhysd/actionlint https://github.com/rhysd/actionlint
  brew install actionlint shellcheck postgresql

start-postgresql-homebrew:
	brew services start postgresql

push remote: ci
	! git branch | grep '* master'
	git push {{remote}}

pr: ci
	gh pr create --web

done remote branch=`git rev-parse --abbrev-ref HEAD`:
	git checkout master
	git diff --no-ext-diff --quiet --exit-code
	git pull --rebase {{remote}} master
	git diff --no-ext-diff --quiet --exit-code {{branch}}
	git branch -D {{branch}}

deploy host:
  cargo run --package deploy -- --host {{host}}

actionlint:
  actionlint

test-on-vagrant: && (deploy '10.9.8.7')
  ssh-keygen -f ~/.ssh/known_hosts -R 10.9.8.7
  vagrant up
  ssh-keyscan 10.9.8.7 >> ~/.ssh/known_hosts
