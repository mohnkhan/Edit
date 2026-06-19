// Test-only misbehaving plugin: never returns from highlight().
// The host's on_progress deadline (50 ms) must abort this and disable the plugin.
fn highlight(line, ext) {
    loop {
        // spin forever
    }
}
