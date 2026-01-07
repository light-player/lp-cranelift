# Phase 6: Update main.rs to show output sections and debug panel

## Goal

Update main.rs to:
1. Show output sections: one per output, displaying LEDs assuming RGB order
2. Show debug panel: textures, shaders, fixtures with actual runtime data

## Tasks

1. Remove `FwHostApp` wrapper, use `LpApp` directly
2. Get `HostOutputProvider` instance to access outputs
3. In UI update loop:
   - Get all outputs from `HostOutputProvider`
   - For each output, create a section showing LEDs (use existing `render_leds()`)
   - Show debug panel with textures, shaders, fixtures
4. Get project config and runtime from `LpApp` for debug UI
5. Update debug UI calls to pass runtime data

## Success Criteria

- Output sections show LEDs for each output (RGB order)
- Debug panel shows actual runtime data
- Code compiles without warnings
- UI displays correctly

