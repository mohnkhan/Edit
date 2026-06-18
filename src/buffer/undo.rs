//! Task T015: UndoStack — a linear undo/redo history for [`super::rope::EditorRope`].
//!
//! The stack maintains a cursor into a `Vec<EditOp>`.  Everything at or after
//! the cursor is "redo-able"; everything before it is "undo-able".  Pushing a
//! new operation truncates the redo branch so the history remains linear.

#![allow(dead_code)]

use super::rope::EditorRope;

// ---------------------------------------------------------------------------
// Public type aliases
// ---------------------------------------------------------------------------

/// A position in the rope expressed as a Unicode char index.
pub type CharIdx = usize;

// ---------------------------------------------------------------------------
// EditOp
// ---------------------------------------------------------------------------

/// A single reversible editing operation.
#[derive(Debug, Clone)]
pub enum EditOp {
    /// Text was inserted at `at`; undo by deleting `text.chars().count()` chars.
    Insert { at: CharIdx, text: String },
    /// Text was deleted from `at`; undo by inserting `text` back at `at`.
    Delete { at: CharIdx, text: String },
    /// A batch of operations applied atomically.  Undo applies each inverse
    /// in **reverse** order.
    Composite(Vec<EditOp>),
}

impl EditOp {
    /// Apply this operation to `rope` (forward / redo direction).
    fn apply(&self, rope: &mut EditorRope) {
        match self {
            EditOp::Insert { at, text } => {
                rope.insert_str(*at, text);
            }
            EditOp::Delete { at, text } => {
                let len = text.chars().count();
                rope.delete_range(*at..*at + len);
            }
            EditOp::Composite(ops) => {
                for op in ops {
                    op.apply(rope);
                }
            }
        }
    }

    /// Apply the **inverse** of this operation to `rope` (undo direction).
    fn apply_inverse(&self, rope: &mut EditorRope) {
        match self {
            // Inverse of Insert: delete the inserted text.
            EditOp::Insert { at, text } => {
                let len = text.chars().count();
                rope.delete_range(*at..*at + len);
            }
            // Inverse of Delete: reinsert the deleted text.
            EditOp::Delete { at, text } => {
                rope.insert_str(*at, text);
            }
            // Inverse of Composite: apply each inverse in reverse order.
            EditOp::Composite(ops) => {
                for op in ops.iter().rev() {
                    op.apply_inverse(rope);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// UndoStack
// ---------------------------------------------------------------------------

/// Linear undo/redo history.
///
/// ```text
/// ops: [ op0  op1  op2 | op3  op4 ]
///                       ^
///                    cursor   (ops[cursor..] are the redo branch)
/// ```
pub struct UndoStack {
    ops: Vec<EditOp>,
    cursor: usize,
}

impl UndoStack {
    // -----------------------------------------------------------------------
    // Construction
    // -----------------------------------------------------------------------

    /// Create an empty undo stack.
    pub fn new() -> Self {
        UndoStack {
            ops: Vec::new(),
            cursor: 0,
        }
    }

    // -----------------------------------------------------------------------
    // Mutation
    // -----------------------------------------------------------------------

    /// Record a new operation.
    ///
    /// Truncates any redo branch (ops at or after the cursor) before pushing,
    /// so history always remains linear after a new edit.
    pub fn push(&mut self, op: EditOp) {
        self.truncate_redo();
        self.ops.push(op);
        self.cursor += 1;
    }

    // -----------------------------------------------------------------------
    // Undo / redo
    // -----------------------------------------------------------------------

    /// Undo the most recent operation.
    ///
    /// Decrements the cursor and applies the inverse of `ops[cursor]` to
    /// `rope`.  Returns the un-done op, or `None` if there is nothing to undo.
    pub fn undo(&mut self, rope: &mut EditorRope) -> Option<EditOp> {
        if self.cursor == 0 {
            return None;
        }
        self.cursor -= 1;
        let op = self.ops[self.cursor].clone();
        op.apply_inverse(rope);
        Some(op)
    }

    /// Redo the next operation in the redo branch.
    ///
    /// Applies `ops[cursor]` to `rope` and increments the cursor.  Returns
    /// the re-done op, or `None` if there is nothing to redo.
    pub fn redo(&mut self, rope: &mut EditorRope) -> Option<EditOp> {
        if self.cursor >= self.ops.len() {
            return None;
        }
        let op = self.ops[self.cursor].clone();
        op.apply(rope);
        self.cursor += 1;
        Some(op)
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    /// Drop all ops at or after the current cursor (the entire redo branch).
    pub fn truncate_redo(&mut self) {
        self.ops.truncate(self.cursor);
    }

    /// Returns `true` if there are no operations recorded at all.
    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

    /// Number of operations that can currently be undone.
    pub fn undo_depth(&self) -> usize {
        self.cursor
    }

    /// Number of operations that can currently be redone.
    pub fn redo_depth(&self) -> usize {
        self.ops.len().saturating_sub(self.cursor)
    }
}

// ---------------------------------------------------------------------------
// Default
// ---------------------------------------------------------------------------

impl Default for UndoStack {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn rope_with(s: &str) -> EditorRope {
        EditorRope::from_str(s)
    }

    // -----------------------------------------------------------------------
    // Basic push / undo / redo
    // -----------------------------------------------------------------------

    #[test]
    fn push_increases_undo_depth() {
        let mut stack = UndoStack::new();
        assert!(stack.is_empty());
        stack.push(EditOp::Insert {
            at: 0,
            text: "a".into(),
        });
        assert_eq!(stack.undo_depth(), 1);
        assert_eq!(stack.redo_depth(), 0);
    }

    #[test]
    fn undo_insert_deletes_text() {
        let mut rope = rope_with("");
        let mut stack = UndoStack::new();

        // Apply and record an Insert.
        rope.insert_str(0, "hello");
        stack.push(EditOp::Insert {
            at: 0,
            text: "hello".into(),
        });

        let undone = stack.undo(&mut rope);
        assert!(undone.is_some());
        assert_eq!(rope.to_string(), "");
    }

    #[test]
    fn undo_delete_reinserts_text() {
        let mut rope = rope_with("hello");
        let mut stack = UndoStack::new();

        // Apply and record a Delete.
        rope.delete_range(0..5);
        stack.push(EditOp::Delete {
            at: 0,
            text: "hello".into(),
        });

        stack.undo(&mut rope);
        assert_eq!(rope.to_string(), "hello");
    }

    #[test]
    fn redo_reapplies_insert() {
        let mut rope = rope_with("");
        let mut stack = UndoStack::new();

        rope.insert_str(0, "hello");
        stack.push(EditOp::Insert {
            at: 0,
            text: "hello".into(),
        });

        stack.undo(&mut rope); // rope = ""
        stack.redo(&mut rope); // rope = "hello"
        assert_eq!(rope.to_string(), "hello");
    }

    #[test]
    fn redo_reapplies_delete() {
        let mut rope = rope_with("hello");
        let mut stack = UndoStack::new();

        rope.delete_range(0..5);
        stack.push(EditOp::Delete {
            at: 0,
            text: "hello".into(),
        });

        stack.undo(&mut rope); // rope = "hello"
        stack.redo(&mut rope); // rope = ""
        assert_eq!(rope.to_string(), "");
    }

    // -----------------------------------------------------------------------
    // Redo branch truncation
    // -----------------------------------------------------------------------

    #[test]
    fn push_after_undo_clears_redo() {
        let mut rope = rope_with("");
        let mut stack = UndoStack::new();

        rope.insert_str(0, "a");
        stack.push(EditOp::Insert { at: 0, text: "a".into() });

        rope.insert_str(1, "b");
        stack.push(EditOp::Insert { at: 1, text: "b".into() });

        stack.undo(&mut rope); // undo "b"
        assert_eq!(stack.redo_depth(), 1);

        // New edit should clear the redo branch.
        rope.insert_str(1, "c");
        stack.push(EditOp::Insert { at: 1, text: "c".into() });

        assert_eq!(stack.redo_depth(), 0);
        assert_eq!(stack.undo_depth(), 2);
    }

    // -----------------------------------------------------------------------
    // Composite
    // -----------------------------------------------------------------------

    #[test]
    fn composite_undo_reverses_all() {
        let mut rope = rope_with("");
        let mut stack = UndoStack::new();

        // Simulate: insert "ab", then delete "b" as one atomic composite.
        rope.insert_str(0, "ab");
        rope.delete_range(1..2);
        // rope = "a"

        stack.push(EditOp::Composite(vec![
            EditOp::Insert { at: 0, text: "ab".into() },
            EditOp::Delete { at: 1, text: "b".into() },
        ]));

        stack.undo(&mut rope);
        // Should undo Delete first (→ reinsert "b" → "ab"), then undo Insert (→ delete "ab" → "").
        assert_eq!(rope.to_string(), "");
    }

    #[test]
    fn composite_redo_replays_all() {
        let mut rope = rope_with("");
        let mut stack = UndoStack::new();

        rope.insert_str(0, "ab");
        rope.delete_range(1..2);

        stack.push(EditOp::Composite(vec![
            EditOp::Insert { at: 0, text: "ab".into() },
            EditOp::Delete { at: 1, text: "b".into() },
        ]));

        stack.undo(&mut rope); // rope = ""
        stack.redo(&mut rope); // rope = "a"
        assert_eq!(rope.to_string(), "a");
    }

    // -----------------------------------------------------------------------
    // Edge cases
    // -----------------------------------------------------------------------

    #[test]
    fn undo_empty_stack_returns_none() {
        let mut rope = rope_with("x");
        let mut stack = UndoStack::new();
        assert!(stack.undo(&mut rope).is_none());
    }

    #[test]
    fn redo_at_top_returns_none() {
        let mut rope = rope_with("");
        let mut stack = UndoStack::new();
        rope.insert_str(0, "x");
        stack.push(EditOp::Insert { at: 0, text: "x".into() });
        // No undo done — redo branch is empty.
        assert!(stack.redo(&mut rope).is_none());
    }
}
