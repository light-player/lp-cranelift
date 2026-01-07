# Phase 5: Move message handling from app.rs to main.rs

## Goal

Move message handling logic from `app.rs` to `main.rs` (or a helper module if needed).

## Tasks

1. Read message handling code from `app.rs` (`handle_command()`, `tick()`, `process_messages()`)
2. Create helper function in main.rs (or separate module) to:
   - Read messages from transport
   - Convert to `MsgIn` (parse commands)
   - Return `Vec<MsgIn>`
3. Update main.rs update loop to:
   - Call helper to get messages
   - Call `LpApp::tick()` with messages
   - Handle `MsgOut` responses (if any)

## Success Criteria

- Message handling is in main.rs (or helper module)
- Transport -> MsgIn conversion works
- `LpApp::tick()` is called with messages
- Code compiles without warnings

