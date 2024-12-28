#!/bin/bash

# Run formatter
echo Formatting...
cargo fmt

# Run linter
echo Linting...
cargo clippy

# Run linter on examples
echo Linting examples...
cargo +nightly clippy --examples

# tests
echo Running tests...
cargo nextest run
echo Running doctests...
cargo test --doc

# generate readme
cargo rdme