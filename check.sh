#!/usr/bin/env bash

set -Eeuo pipefail

cleanup() {
    local return_code=$?
    exit $return_code
}

trap cleanup EXIT

echo_and_run() { echo "$ ${*@Q}"; "$@"; }

echo_and_run cargo +nightly fmt --all -- --check
echo_and_run cargo outdated --exit-code 1

for flags in "--no-default-features" "" "--all-features"; do
    echo_and_run cargo clippy --workspace --all-targets $flags -- -D warnings
    echo_and_run cargo build --workspace --all-targets $flags
    echo_and_run cargo test --workspace --all-targets $flags
    echo_and_run cargo test --workspace --doc $flags
done

echo_and_run cargo deny --workspace --all-features check

echo "All checks succeeded." 1>&2
