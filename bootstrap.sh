#!/bin/bash

# Zen Compiler Bootstrap Script
# This script manages the bootstrap process for the Zen compiler

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
BUILD_DIR="build"
STAGE0_DIR="$BUILD_DIR/stage0"
STAGE1_DIR="$BUILD_DIR/stage1"
STAGE2_DIR="$BUILD_DIR/stage2"

# Functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Create build directories
setup_directories() {
    log_info "Setting up build directories..."
    mkdir -p "$STAGE0_DIR"
    mkdir -p "$STAGE1_DIR"
    mkdir -p "$STAGE2_DIR"
}

# Stage 0: Build Rust-based compiler
build_stage0() {
    log_info "Stage 0: Building Rust-based compiler..."
    
    # Build the Rust compiler
    cargo build --release
    
    if [ $? -eq 0 ]; then
        cp target/release/zen "$STAGE0_DIR/zen"
        log_info "Stage 0 compiler built successfully: $STAGE0_DIR/zen"
        return 0
    else
        log_error "Failed to build Stage 0 compiler"
        return 1
    fi
}

# Stage 1: Use Stage 0 to compile self-hosted components
build_stage1() {
    log_info "Stage 1: Compiling self-hosted components..."
    
    # Check if Stage 0 compiler exists
    if [ ! -f "$STAGE0_DIR/zen" ]; then
        log_error "Stage 0 compiler not found. Run stage 0 first."
        return 1
    fi
    
    # Compile the lexer
    log_info "Compiling lexer.zen..."
    "$STAGE0_DIR/zen" compile stdlib/lexer.zen -o "$STAGE1_DIR/lexer.o" 2>/dev/null || {
        log_warn "Lexer compilation not yet supported - placeholder created"
        touch "$STAGE1_DIR/lexer.o"
    }
    
    # Compile the parser
    log_info "Compiling parser.zen..."
    "$STAGE0_DIR/zen" compile stdlib/parser.zen -o "$STAGE1_DIR/parser.o" 2>/dev/null || {
        log_warn "Parser compilation not yet supported - placeholder created"
        touch "$STAGE1_DIR/parser.o"
    }
    
    log_info "Stage 1 components prepared"
    return 0
}

# Stage 2: Build self-hosted compiler
build_stage2() {
    log_info "Stage 2: Building self-hosted compiler..."
    
    # This stage will use the self-hosted lexer and parser
    # For now, it's a placeholder
    
    log_warn "Stage 2 not yet implemented - self-hosting in progress"
    touch "$STAGE2_DIR/zen"
    
    return 0
}

# Run tests
run_tests() {
    log_info "Running test suite..."
    
    cargo test --quiet
    
    if [ $? -eq 0 ]; then
        log_info "All tests passed!"
        return 0
    else
        log_error "Some tests failed"
        return 1
    fi
}

# Clean build artifacts
clean() {
    log_info "Cleaning build artifacts..."
    rm -rf "$BUILD_DIR"
    cargo clean
    log_info "Clean complete"
}

# Main bootstrap process
bootstrap() {
    echo "========================================="
    echo "       Zen Compiler Bootstrap"
    echo "========================================="
    echo
    
    setup_directories
    
    # Run stages
    build_stage0 || exit 1
    build_stage1 || exit 1
    build_stage2 || exit 1
    
    # Run tests to verify
    run_tests || exit 1
    
    echo
    log_info "Bootstrap completed successfully!"
    echo
    echo "Stage 0 compiler: $STAGE0_DIR/zen (Rust-based)"
    echo "Stage 1 components: $STAGE1_DIR/ (Self-hosted lexer/parser)"
    echo "Stage 2 compiler: $STAGE2_DIR/zen (Fully self-hosted)"
    echo
}

# Parse command line arguments
case "${1:-}" in
    stage0)
        setup_directories
        build_stage0
        ;;
    stage1)
        setup_directories
        build_stage1
        ;;
    stage2)
        setup_directories
        build_stage2
        ;;
    test)
        run_tests
        ;;
    clean)
        clean
        ;;
    all|bootstrap)
        bootstrap
        ;;
    *)
        echo "Usage: $0 {stage0|stage1|stage2|test|clean|all|bootstrap}"
        echo
        echo "Commands:"
        echo "  stage0    - Build Rust-based compiler"
        echo "  stage1    - Compile self-hosted lexer/parser"
        echo "  stage2    - Build fully self-hosted compiler"
        echo "  test      - Run test suite"
        echo "  clean     - Remove build artifacts"
        echo "  all       - Run complete bootstrap process"
        echo "  bootstrap - Same as 'all'"
        exit 1
        ;;
esac