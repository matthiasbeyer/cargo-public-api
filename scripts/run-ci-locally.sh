#!/usr/bin/env bash
set -o errexit -o nounset -o pipefail -o xtrace

# This script tries to emulate a run of CI.yml. If you can run this script
# without errors you can be reasonably sure that CI will pass for real when you
# push the code.

cargo fmt -- --check

RUSTDOCFLAGS='--deny warnings' cargo doc --locked --no-deps

cargo clippy --locked --all-targets --all-features -- -D clippy::all -D clippy::pedantic

cargo test --locked

./scripts/test-invocation-variants.sh
