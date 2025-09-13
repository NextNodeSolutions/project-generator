.PHONY: format lint fix dev test clean

# Auto-fix formatting and linting issues
fix: format lint

# Format code automatically
format:
	cargo fmt

# Lint and auto-fix clippy issues
lint:
	cargo clippy --fix --allow-dirty --allow-staged

# Development workflow - format, lint, then test
dev: fix test

# Run tests
test:
	cargo test

# Build release
build:
	cargo build --release

# Clean build artifacts
clean:
	cargo clean