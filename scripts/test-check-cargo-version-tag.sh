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

run_failure() {
	expected_status=$1
	output_file=$2
	shift 2

	set +e
	"$@" >"${output_file}" 2>&1
	status=$?
	set -e

	[ "${status}" -eq "${expected_status}" ]
}

mismatch_output="${tmpdir}/mismatch.out"
run_failure 1 "${mismatch_output}" env CARGO_MANIFEST_PATH="${manifest}" scripts/check-cargo-version-tag.sh v1.2.4
grep "does not match Cargo.toml version '1.2.3'" "${mismatch_output}"

missing_tag_output="${tmpdir}/missing-tag.out"
run_failure 2 "${missing_tag_output}" env CARGO_MANIFEST_PATH="${manifest}" GITHUB_REF_NAME= scripts/check-cargo-version-tag.sh
grep 'release tag name is required' "${missing_tag_output}"
