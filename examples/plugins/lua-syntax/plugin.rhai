// Lua syntax highlighter — reference plugin for the `edit` Plugin API (Feature 008).
//
// The host calls `highlight(line, ext)` once per visible line and expects an array of
// token maps: #{ start, end, kind } where start/end are BYTE offsets into `line` (end
// exclusive) and kind is one of:
//   "default" "keyword" "string" "comment" "number" "operator" "type"
//
// Offsets are byte offsets; for ASCII source (the common case) they equal character
// indices. The host validates every token (bounds, no overlap, char boundaries) and
// silently discards the whole array if any token is invalid.

fn is_word_char(ch) {
    if ch == "_" { return true; }
    if ch >= "a" && ch <= "z" { return true; }
    if ch >= "A" && ch <= "Z" { return true; }
    if ch >= "0" && ch <= "9" { return true; }
    false
}

fn is_number(w) {
    if w.len() == 0 { return false; }
    let k = 0;
    while k < w.len() {
        let d = w.sub_string(k, 1);
        if d < "0" || d > "9" { return false; }
        k += 1;
    }
    true
}

fn highlight(line, ext) {
    let tokens = [];
    let n = line.len();

    // A "--" begins a line comment that runs to end of line.
    let c = line.index_of("--");
    let limit = if c >= 0 {
        tokens.push(#{ start: c, end: n, kind: "comment" });
        c
    } else {
        n
    };

    let keywords = [
        "function", "local", "end", "if", "then", "else", "elseif",
        "for", "while", "do", "return", "nil", "true", "false",
        "and", "or", "not", "in", "repeat", "until", "break",
    ];

    // Scan words in the code portion [0, limit).
    let i = 0;
    while i < limit {
        let ch = line.sub_string(i, 1);
        if is_word_char(ch) {
            let j = i;
            while j < limit && is_word_char(line.sub_string(j, 1)) {
                j += 1;
            }
            let word = line.sub_string(i, j - i);
            if keywords.contains(word) {
                tokens.push(#{ start: i, end: j, kind: "keyword" });
            } else if is_number(word) {
                tokens.push(#{ start: i, end: j, kind: "number" });
            }
            i = j;
        } else {
            i += 1;
        }
    }

    tokens
}
