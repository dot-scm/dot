# dot CLI Makefile
# A Git proxy for managing hidden directories with version control

.PHONY: help build install uninstall clean test check fmt clippy release dev setup

# Default target
help:
	@echo "dot CLI - Available commands:"
	@echo ""
	@echo "Building:"
	@echo "  build      - Build the project in debug mode"
	@echo "  release    - Build the project in release mode"
	@echo "  dev        - Build and run in development mode"
	@echo ""
	@echo "Installation:"
	@echo "  install    - Install dot to global PATH (release build)"
	@echo "  uninstall  - Remove dot from global PATH"
	@echo ""
	@echo "Development:"
	@echo "  check      - Check code without building"
	@echo "  test       - Run all tests"
	@echo "  fmt        - Format code"
	@echo "  clippy     - Run clippy linter"
	@echo "  clean      - Clean build artifacts"
	@echo ""
	@echo "Setup:"
	@echo "  setup      - Initial setup (install dependencies, create config)"
	@echo ""
	@echo "Usage after installation:"
	@echo "  dot init <dir>     - Initialize hidden directory"
	@echo "  dot status         - Show repository status"
	@echo "  dot add <files>    - Add files to repositories"
	@echo "  dot commit -m msg  - Commit to all repositories"
	@echo "  dot push           - Push all repositories"
	@echo "  dot clone <url>    - Clone with hidden repositories"

# Build targets
build:
	@echo "🔨 Building dot in debug mode..."
	cargo build

release:
	@echo "🚀 Building dot in release mode..."
	cargo build --release

# Development targets
dev: build
	@echo "🔧 Running dot in development mode..."
	@echo "Use: ./target/debug/dot --help"

check:
	@echo "✅ Checking code..."
	cargo check

test:
	@echo "🧪 Running tests..."
	cargo test

fmt:
	@echo "🎨 Formatting code..."
	cargo fmt

clippy:
	@echo "📎 Running clippy..."
	cargo clippy -- -D warnings

clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean

# Installation targets
install: release
	@echo "📦 Installing dot to global PATH..."
	cargo install --path . --force
	@echo ""
	@echo "✅ dot has been installed successfully!"
	@echo ""
	@echo "🔧 Setup required:"
	@echo "1. Set your GitHub token:"
	@echo "   export GITHUB_TOKEN=\"your_github_token_here\""
	@echo ""
	@echo "2. Configure organizations in ~/.dot/dot.conf:"
	@echo "   {"
	@echo "     \"authorized_organizations\": [\"your-org\"],"
	@echo "     \"default_organization\": \"your-org\""
	@echo "   }"
	@echo ""
	@echo "3. Test installation:"
	@echo "   dot --version"
	@echo "   dot --help"

uninstall:
	@echo "🗑️  Uninstalling dot..."
	cargo uninstall dot || echo "dot was not installed via cargo"
	@echo "✅ dot has been uninstalled"
	@echo ""
	@echo "Note: Configuration files in ~/.dot/ are preserved"
	@echo "Remove them manually if needed: rm -rf ~/.dot/"

# Setup and configuration
setup:
	@echo "🛠️  Setting up dot development environment..."
	@echo ""
	@echo "1. Checking Rust installation..."
	@rustc --version || (echo "❌ Rust not found. Install from https://rustup.rs/" && exit 1)
	@echo "✅ Rust is installed"
	@echo ""
	@echo "2. Checking Git installation..."
	@git --version || (echo "❌ Git not found. Install Git first" && exit 1)
	@echo "✅ Git is installed"
	@echo ""
	@echo "3. Installing Rust components..."
	rustup component add rustfmt clippy
	@echo ""
	@echo "4. Building project..."
	$(MAKE) build
	@echo ""
	@echo "5. Running tests..."
	$(MAKE) test
	@echo ""
	@echo "✅ Setup complete!"
	@echo ""
	@echo "Next steps:"
	@echo "- Run 'make install' to install globally"
	@echo "- Set GITHUB_TOKEN environment variable"
	@echo "- Configure ~/.dot/dot.conf with your organizations"

# Quick development workflow
all: fmt clippy test build

# Release workflow
prepare-release: clean fmt clippy test release
	@echo "🎉 Release build ready!"
	@echo "Binary location: ./target/release/dot"
	@echo "Run 'make install' to install globally"

# Development helpers
run-debug: build
	@echo "🏃 Running debug build..."
	./target/debug/dot --help

run-release: release
	@echo "🏃 Running release build..."
	./target/release/dot --help

# Check if dot is installed
check-install:
	@echo "🔍 Checking dot installation..."
	@which dot > /dev/null && echo "✅ dot is installed at: $$(which dot)" || echo "❌ dot is not installed"
	@dot --version 2>/dev/null || echo "❌ dot command not working"

# Create example configuration
create-config:
	@echo "📝 Creating example configuration..."
	@mkdir -p ~/.dot
	@echo '{\n  "authorized_organizations": ["your-org-here"],\n  "default_organization": "your-org-here"\n}' > ~/.dot/dot.conf.example
	@echo "✅ Example config created at ~/.dot/dot.conf.example"
	@echo "Copy and edit it to ~/.dot/dot.conf"

# Show current configuration
show-config:
	@echo "📋 Current dot configuration:"
	@echo "Config file: ~/.dot/dot.conf"
	@if [ -f ~/.dot/dot.conf ]; then \
		echo "Content:"; \
		cat ~/.dot/dot.conf; \
	else \
		echo "❌ Config file not found"; \
		echo "Run 'make create-config' to create an example"; \
	fi
	@echo ""
	@echo "GitHub Token: $${GITHUB_TOKEN:+✅ Set}$${GITHUB_TOKEN:-❌ Not set}"

# Benchmark (if you add benchmarks later)
bench:
	@echo "📊 Running benchmarks..."
	cargo bench

# Documentation
docs:
	@echo "📚 Building documentation..."
	cargo doc --open

# Update dependencies
update:
	@echo "⬆️  Updating dependencies..."
	cargo update

# Security audit
audit:
	@echo "🔒 Running security audit..."
	cargo audit || echo "Install cargo-audit with: cargo install cargo-audit"