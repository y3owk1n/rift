default:
    @just --list

# Build commands
build:
    cargo build

build-release:
    cargo build --release

build-check:
    cargo check --locked

# Universal binary builds
build-universal:
    cargo build --bins --target aarch64-apple-darwin
    cargo build --bins --target x86_64-apple-darwin

build-universal-release:
    cargo build --release --bins --target aarch64-apple-darwin
    cargo build --release --bins --target x86_64-apple-darwin

# Testing
test:
    cargo test --lib -- --test-threads=1

test-one name='':
    cargo test --lib -- --test-threads=1 {{ name }}

bench:
    cargo bench

doc-test:
    cargo test --doc

# Linting and formatting
fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all --check

clippy:
    cargo clippy --all-targets --all-features

clippy-fix:
    cargo clippy --fix --allow-dirty

# All quality checks
check: fmt-check clippy build-check
