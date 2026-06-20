# Quality Checklist: Visible text selection

**Created**: 2026-06-20 · **Feature**: [spec.md](../spec.md)

- [x] CHK001 Is "highlight exactly the ordered range, direction-independent, multi-line" specified? [§FR-001/§FR-008]
- [x] CHK002 Is Select All → whole-buffer highlight required? [§FR-002/§SC-001]
- [x] CHK003 Are Shift+Arrow/Home/End selection semantics defined? [§FR-003/§SC-002]
- [x] CHK004 Is clear-on-(non-shift move / typing / single click) specified? [§FR-004/§FR-006/§SC-004]
- [x] CHK005 Is mouse press/drag/release vs single-click behavior defined? [§FR-006/§SC-003]
- [x] CHK006 Is Copy/Cut-on-selection + undoable stated? [§FR-005]
- [x] CHK007 Is UTF-8/scroll/soft-wrap correctness required (only visible cells, no split)? [§FR-007]
- [x] CHK008 Is the selection highlight required to stay distinct from the search-match highlight? [§FR-001]
- [x] CHK009 Is no-regression to movement/editing/search-highlight/menu-mouse stated? [§FR-009/§SC-005]
- [x] CHK010 Are success criteria objectively measurable (N chars → N selected/copied; drag A→B exact)? [§SC-002/§SC-003]
