# Fix result scrolling lag

- Investigated the result-pane `j`/`k` navigation lag in the TUI.
- Root cause: the UI thread only handled one input event per 100ms render tick and used a bounded UI action channel, which could block on repeated key presses.
- Fix approach: switch the UI action channel to unbounded, increase the UI refresh cadence, and drain all pending crossterm events each frame so held/repeated keys are processed promptly.
