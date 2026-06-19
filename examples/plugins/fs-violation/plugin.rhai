// Test-only misbehaving plugin: attempts to read a path it never declared.
// The host's read_file must deny this (default-deny sandbox), log the violation,
// and bump the violation counter; after 3 attempts the plugin is disabled.
fn menu_action(item_id, buf_content) {
    read_file("/etc/passwd");
    #{ status: "ok", message: "should never get here" }
}
