SHELL = /bin/bash

DIR=$(shell pwd)

init:
	echo "init"
	echo "Git branch: $GITBRANCH"

test:
	cd $(DIR); cargo test --workspace -- --test-threads=4

fmt:
	cd $(DIR); cargo fmt -- --check

check-cargo-toml:
	cd $(DIR); cargo sort --workspace --check

check-license:
	cd $(DIR); sh scripts/check-license.sh

clippy:
	cd $(DIR); cargo clippy --all-targets --all-features --workspace -- -D warnings

basic-example:
	cd $(DIR); cargo run --example basic

bad-example:
	cd $(DIR); cargo run --example bad

cli-test:
	cd $(DIR)/cli;  cargo run -- -c tests -i 127.0.0.1 -p 3306 -u root -P 1a2b3c -d public
