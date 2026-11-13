#!/bin/sh
# Usage: check.sh <adapter src dir> <fuzz targets dir>
# Fails when a type saves a file through save_whole_or_nothing and no fuzz target names it (ADR 0005).
# shortcut: reads top-level src files and one-line, non-generic impl headers only; upgrade when an adapter saves from a submodule or a generic type.
src=$1
targets=$2
status=0

saving_types=$(awk '
  /^impl / { type = $(NF - 1) }
  /save_whole_or_nothing\(/ && !/fn save_whole_or_nothing/ { print type }
' "$src"/*.rs | sort -u)

for type in $saving_types; do
  if ! grep -qw "$type" "$targets"/*.rs; then
    echo "$type saves through save_whole_or_nothing, but no fuzz target reads it"
    status=1
  fi
done

exit $status
