.PHONY: prep test fmt clippy

prep: test fmt clippy

test:
	@echo "Running tests..."
	cargo test --verbose

fmt:
	@echo "Checking formatting..."
	cargo fmt --check

clippy:
	@echo "Running clippy..."
	cargo clippy -- -D warnings

# Helper target to fix formatting
fmt-fix:
	@echo "Fixing formatting..."
	cargo fmt
