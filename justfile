[private]
default:
    @just --list

# Build the code
build:
    @cargo build --workspace

# Test the code
test:
    @cargo test --workspace

# Lint the code
clippy:
    @cargo clippy -- -D warnings

# Publish the code
publish:
