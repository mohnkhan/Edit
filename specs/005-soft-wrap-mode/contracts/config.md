# Contract: Config.soft_wrap

**File**: `src/config/schema.rs`

## Field Addition

```rust
/// Enable soft-wrap rendering: long lines fold at the viewport width
/// instead of scrolling horizontally. Non-DOS extension.
///
/// Default: `false`
#[serde(default)]
pub soft_wrap: bool,
```

## Default Implementation Update

```rust
impl Default for Config {
    fn default() -> Self {
        Self {
            // ... existing fields ...
            soft_wrap: false,
        }
    }
}
```

## TOML Key

```toml
soft_wrap = true   # or false (default when absent)
```

Key `soft_wrap` in `$XDG_CONFIG_HOME/edit/config.toml`.

## Serialization Guarantees

- `#[serde(default)]` ensures existing configs without `soft_wrap` deserialize to `false` without error.
- `soft_wrap` IS serialized on save (unlike runtime-only `#[serde(skip)]` fields).
- The `serde_round_trip_default` test must be updated to include `soft_wrap`.

## Existing Test Update Required

`default_values_match_contract` must assert `assert!(!cfg.soft_wrap)`.
`serde_round_trip_default` must compare `restored.soft_wrap == original.soft_wrap`.
