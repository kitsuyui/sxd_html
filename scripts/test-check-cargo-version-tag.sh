#!/bin/sh
# Exercise release tag and Cargo.toml version matching.

set -eu

tmpdir=$(mktemp -d)
trap 'rm -rf "${tmpdir}"' EXIT

manifest="${tmpdir}/Cargo.toml"
cat >"${manifest}" <<'TOML'
[package]
name = "example"
version = "1.2.3"
TOML

CARGO_MANIFEST_PATH="${manifest}" scripts/check-cargo-version-tag.sh v1.2.3
CARGO_MANIFEST_PATH="${manifest}" scripts/check-cargo-version-tag.sh 1.2.3
CARGO_MANIFEST_PATH="${manifest}" scripts/check-cargo-version-tag.sh refs/tags/v1.2.3

set +e
output=$(CARGO_MANIFEST_PATH="${manifest}" scripts/check-cargo-version-tag.sh v1.2.4 2>&1)
status=$?
set -e
[ "${status}" -eq 1 ]
printf '%s\n' "${output}" | grep "does not match Cargo.toml version '1.2.3'"

set +e
output=$(CARGO_MANIFEST_PATH="${manifest}" scripts/check-cargo-version-tag.sh 2>&1)
status=$?
set -e
[ "${status}" -eq 2 ]
printf '%s\n' "${output}" | grep 'release tag name is required'
