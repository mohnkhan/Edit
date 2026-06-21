# Feature Specification: Per-Pane Wrap Cache for Split View

**Feature Branch**: `048-per-pane-wrap-cache` | **Created**: 2026-06-21 | **Status**: Draft
**Input**: Issue #80 — "Per-pane wrap cache for split view (follow-up to #76)."

## Overview

After per-tab soft-wrap (044), split view still uses a single wrap cache for the active buffer. Two
consequences: a wrapped **non-active** pane has no cache and falls back to rendering **unwrapped** (it
ignores its own wrap setting), and the active pane's cache is computed at the **full** terminal width
rather than its half-width pane — so wrap points and the vertical scrollbar are off in split view. This
feature makes wrap correct **per pane**: each visible pane wraps at its own width with its own cache.

It is behavior-preserving for single (non-split) view — there the one pane already spans the content
width. No user-facing controls change; split panes simply render their wrap correctly.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Both split panes wrap at their own width (Priority: P1)
As a user in vertical split with soft-wrap on, each pane wraps long lines to its own (half) width — the
active pane and the other pane both — with a correct line-number gutter and vertical scrollbar.

**Independent Test**: In split view with both buffers wrapped, render and assert each pane's wrap layout
matches its own pane width (not full width), and the non-active pane is wrapped (not unwrapped).

**Acceptance Scenarios**:
1. **Given** split view, both tabs wrapped, **When** rendered, **Then** the non-active pane is wrapped
   (honors its setting), not shown unwrapped.
2. **Given** split view, **When** a pane wraps, **Then** it wraps at its pane width, and its vertical
   scrollbar's total reflects that pane's wrapped row count.
3. **Given** one pane wrapped and the other not, **When** rendered, **Then** each pane reflects its own
   setting independently.

### User Story 2 - Single view unchanged (Priority: P1)
Single-pane view behaves exactly as before (one cache at the content width).

**Acceptance Scenarios**:
1. **Given** single view, **When** wrap is on, **Then** wrapping/scrollbar are identical to before.

### Edge Cases
- Switching the active tab in split → caches recompute for the now-active and now-other panes (no stale
  cache from before the switch; extends 043 invalidation).
- A pane too narrow for wrap behaves like the existing narrow guard (no panic).
- Only one buffer / not split → no alt cache.

## Requirements *(mandatory)*
- **FR-001**: In split view, each visible pane's wrap MUST be computed at that pane's own content width.
- **FR-002**: A wrapped non-active split pane MUST render wrapped (using its own cache), not fall back to
  unwrapped.
- **FR-003**: Each pane's vertical scrollbar total MUST reflect that pane's wrapped row count.
- **FR-004**: Single-view rendering MUST be unchanged (behavior-preserving).
- **FR-005**: Caches MUST stay correct across tab switches and edits (invalidate/recompute; no stale
  cross-pane cache), extending feature 043.
- **FR-006**: No panic for any split/wrap/size combination; the 042 fuzz + 046 index guards hold; full
  suite + `clippy -D warnings` + `fmt` clean.

## Success Criteria *(mandatory)*
- **SC-001**: In split view both wrapped panes wrap at half width (test on rendered output / cache).
- **SC-002**: The non-active wrapped pane is wrapped, not unwrapped (test).
- **SC-003**: Single-view output is byte-identical to before for the same content (regression test).
- **SC-004**: Full suite + lints green; fuzz (which toggles wrap + switches) still passes.

## Assumptions
- Max two visible panes (vertical split), so a primary + one alt cache suffices (no general N-cache).
- The two panes are equal-or-±1 width; each cache uses its own pane content width.
- `content_width()` becomes split-aware (active pane width); a sibling computes the other pane's width.
