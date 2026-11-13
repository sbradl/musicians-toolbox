#!/bin/sh
# Usage: check.sh <root Cargo.toml> <copy Cargo.toml>
# Fails when the copy's lint tables differ from the root workspace's (user decision: fuzz crate uses the same rules).
root=$1
copy=$2
lint_tables_root=$(mktemp)
lint_tables_copy=$(mktemp)
trap 'rm -f "$lint_tables_root" "$lint_tables_copy"' EXIT

lint_tables() {
  awk '/^\[/ { in_lints = /^\[workspace\.lints\./ } in_lints && NF' "$1"
}

lint_tables "$root" > "$lint_tables_root"
lint_tables "$copy" > "$lint_tables_copy"

if ! differences=$(diff -U0 "$lint_tables_root" "$lint_tables_copy"); then
  echo "$copy has other lint rules than $root:"
  echo "$differences" | tail -n +3
  exit 1
fi
