# Add Ctrl+U / Ctrl+D scroll support

## Summary
- Added new UI actions `ScrollHalfUp` and `ScrollHalfDown`.
- Wired keyboard shortcuts:
  - `Ctrl+d` => scroll down by 10 lines
  - `Ctrl+u` => scroll up by 10 lines
- Updated help overlay keybinding text.

## Notes
- Implemented as fixed 10-line jumps because UI action handling does not currently receive viewport height.
- Scroll movement is bounded safely with saturating math to avoid underflow/overflow.
