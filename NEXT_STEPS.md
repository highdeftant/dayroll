# Dayroll Next Steps

## Priority Order

1. Search/filter behavior
   - Decide whether search is a real first-class mode with explicit entry/exit.
   - Otherwise remove the implicit type-anywhere filtering behavior.

2. Documentation sync
   - Update README keybindings.
   - Update add/edit/move/delete flow.
   - Update storage format section.
   - Update search behavior notes.

3. Core test coverage
   - Add tests for add, edit, move, toggle complete, and delete.
   - Add save/load roundtrip coverage.
   - Add month-shift edge cases.

4. Cleanup
   - Remove thin compatibility wrappers.
   - Remove noisy scaffolding that no longer buys anything.

5. UI polish
   - Add a clear-search key.
   - Improve empty-state hints.
   - Tighten form ergonomics and field hints.

## Notes

- Keep the TUI behavior stable while tightening internals.
- Prefer small, reversible changes.
- Validate each step with tests before moving on.
