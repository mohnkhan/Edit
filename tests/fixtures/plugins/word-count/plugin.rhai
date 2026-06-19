// Word-count menu plugin — reference plugin for the `edit` Plugin API (Feature 008).
//
// The host calls `menu_action(item_id, buf_content)` when the user activates the menu item
// declared in plugin.toml. It returns #{ status, message } and/or calls the host
// `status_bar(msg)` function. Here we count whitespace-separated words.

fn is_space(ch) {
    ch == " " || ch == "\t" || ch == "\n" || ch == "\r"
}

fn menu_action(item_id, buf_content) {
    let count = 0;
    let in_word = false;
    let n = buf_content.len();
    let i = 0;
    while i < n {
        let ch = buf_content.sub_string(i, 1);
        if is_space(ch) {
            in_word = false;
        } else if !in_word {
            in_word = true;
            count += 1;
        }
        i += 1;
    }
    let msg = "Word count: " + count;
    status_bar(msg);
    #{ status: "ok", message: msg }
}
