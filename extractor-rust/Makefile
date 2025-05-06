MAIN_BINARY_NAME=vivarium_assistant

.PHONY: ci
ci: test-verbose clippy build-release fix fmt check-repository-unchanged

.PHONY: cleanup
cleanup: fix clippy fmt

.PHONY: test
test:
	cargo test

.PHONY: test-verbose
test-verbose:
	RUST_BACKTRACE=1 cargo test -- --nocapture

.PHONY: clippy
clippy:
	cargo clippy -- -D warnings

.PHONY: fmt
fmt:
	cargo fmt

.PHONY: fix
fix:
	cargo fix --allow-staged --allow-dirty
	cargo clippy --fix --allow-staged --allow-dirty
	cargo fmt

.PHONY: build-release
build-release:
	cargo build --bin $(MAIN_BINARY_NAME) --release

.PHONY: run
run:
	RUST_LOG=debug cargo run --bin $(MAIN_BINARY_NAME) example_config.toml

.PHONY: check-repository-unchanged
check-repository-unchanged:
	./tools/check_repository_unchanged.sh

