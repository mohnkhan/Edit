// Returns overlapping tokens; the host must discard the whole array but keep the plugin enabled.
fn highlight(line, ext) {
    [ #{ start: 0, end: 5, kind: "keyword" }, #{ start: 3, end: 8, kind: "string" } ]
}
