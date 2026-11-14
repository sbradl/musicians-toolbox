#!/bin/sh
# Reads a `cargo llvm-cov --show-missing-lines` report; lists every missed line not marked `// coverage: <reason>` (ADR 0005).
awk -F': ' '
/\.rs: [0-9]/ {
    file = $1
    count = split($2, numbers, ", ")
    delete source
    at = 0
    while ((getline text < file) > 0) source[++at] = text
    close(file)
    for (i = 1; i <= count; i++) {
        number = numbers[i] + 0
        if (source[number] !~ /\/\/ coverage: [^ ]/) {
            print file ":" number ": untested and not marked // coverage: <reason>"
            unmarked = 1
        }
    }
}
END { exit unmarked }
' "$1"
