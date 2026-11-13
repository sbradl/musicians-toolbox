# Finds an `if` or `return` that needs a blank line above it (CLAUDE.md, Readable code).
FNR == 1 { above = "" }

{
    line = $0
    sub(/^[ \t]+/, "", line)
}

line ~ /^(\/\/|\/\*|\*|#\[|@)/ { next }

line ~ /^(if[ (]|return([ ;(]|$))/ && !may_follow(above) {
    word = line
    sub(/[ ;(].*$/, "", word)
    printf "%s:%d: blank line needed before `%s`\n", FILENAME, FNR, word
    found = 1
}

{ above = line }

END { exit found }

function may_follow(text) {
    return text == "" || text ~ /[{(\[=,>:]$/
}
