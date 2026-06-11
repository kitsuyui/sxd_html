#!/bin/sh
# Ensure a release tag publishes the same version as Cargo.toml.

set -eu

manifest_path=${CARGO_MANIFEST_PATH:-Cargo.toml}
tag_name=${1:-${GITHUB_REF_NAME:-}}

if [ -z "${tag_name}" ]; then
	echo "Error: release tag name is required" >&2
	exit 2
fi

case "${tag_name}" in
refs/tags/*)
	tag_name=${tag_name#refs/tags/}
	;;
esac

tag_version=${tag_name#v}

cargo_version=$(
	awk '
    /^\[package\]/ {
      in_package = 1
      next
    }
    /^\[/ && in_package {
      in_package = 0
    }
    in_package && $1 == "version" && $2 == "=" {
      gsub(/^"/, "", $3)
      gsub(/"$/, "", $3)
      print $3
      exit
    }
  ' "${manifest_path}"
)

if [ -z "${cargo_version}" ]; then
	echo "Error: package.version not found in ${manifest_path}" >&2
	exit 1
fi

if [ "${cargo_version}" != "${tag_version}" ]; then
	echo "Error: release tag '${tag_name}' does not match Cargo.toml version '${cargo_version}'" >&2
	exit 1
fi

echo "Release tag '${tag_name}' matches Cargo.toml version '${cargo_version}'."
