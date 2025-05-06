MAIN_BINARY_NAME=extractor-rust

.PHONY: ci
ci: test-verbose clippy build-release fix fmt

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
