# Zen Language Makefile

.PHONY: all build test clean install format check lint help safe-build mem-watch

all: build

build:
	@echo "Building Zen compiler..."
	@cargo build --release
	@echo "✓ Compiler built"

debug:
	@echo "Building Zen compiler (debug)..."
	@cargo build
	@echo "✓ Debug build complete"

test:
	@echo "Running tests..."
	@cargo test --all
	@echo "✓ Tests complete"

install: build
	@echo "Installing Zen compiler..."
	@cargo install --path .
	@echo "✓ Installed"

format:
	@cargo fmt
	@echo "✓ Formatted"

check:
	@cargo check --all-targets
	@echo "✓ Check complete"

lint:
	@./scripts/lint.sh

clean:
	@cargo clean
	@rm -f *.zen.out
	@echo "✓ Clean"

docs:
	@cargo doc --no-deps --open

safe-build:
	@./scripts/safe-build.sh build

mem-watch:
	@./scripts/mem-watch.sh

release: clean
	@cargo build --release
	@strip target/release/zen
	@ls -lh target/release/zen

help:
	@echo "Zen Language Build System"
	@echo ""
	@echo "  make build      - Build compiler (release)"
	@echo "  make debug      - Build compiler (debug)"
	@echo "  make test       - Run all tests"
	@echo "  make install    - Install compiler"
	@echo "  make format     - Format code"
	@echo "  make check      - Check without building"
	@echo "  make lint       - Run clippy"
	@echo "  make clean      - Clean artifacts"
	@echo "  make docs       - Build docs"
	@echo "  make safe-build - Build with OOM protection"
	@echo "  make mem-watch  - Monitor memory"
	@echo "  make release    - Build stripped release"
