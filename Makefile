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

cli-test:
	cd $(DIR)/sqlness-cli;  cargo run -- -t mysql -c tests/mysql -i 127.0.0.1 -p 3306 -u root -P 1a2b3c -d public
	cd $(DIR)/sqlness-cli;  cargo run -- -t postgresql -c tests/postgresql -i 127.0.0.1 -p 5432 -u postgres -P postgres -d postgres

example: good-example bad-example

good-example: basic-example interceptor-arg-example interceptor-replace-example interceptor-sort-result-example interceptor-env-example

basic-example:
	cd $(DIR)/sqlness; cargo run --example basic

bad-example:
	cd $(DIR)/sqlness; cargo run --example bad

interceptor-example:
	cd $(DIR)/sqlness; cargo run --example interceptor
