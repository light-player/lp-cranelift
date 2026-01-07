# Phase 3: Modify HostOutputProvider to track outputs in HashMap

## Goal

Modify `HostOutputProvider` to track created outputs in a HashMap so they can be accessed for UI rendering.

## Tasks

1. Add field `outputs: HashMap<OutputId, Arc<Mutex<HostLedOutput>>>` to `HostOutputProvider`
2. Modify `create_output()` to:
   - Create `HostLedOutput` as before
   - Wrap in `Arc<Mutex<>>`
   - Store in HashMap with OutputId (need to pass OutputId to `create_output()`)
   - Return the `Arc<Mutex<HostLedOutput>>` wrapped as `Box<dyn LedOutput>` (or change return type)
3. Add `get_output(&self, id: OutputId) -> Option<Arc<Mutex<HostLedOutput>>>`
4. Add `get_all_outputs(&self) -> &HashMap<OutputId, Arc<Mutex<HostLedOutput>>>`
5. Update `OutputProvider` trait if needed (may need to pass OutputId)

**Note:** We may need to modify the `OutputProvider` trait to pass `OutputId` to `create_output()`, or we can track outputs separately by matching configs. Let's check how `create_output()` is called.

## Success Criteria

- `HostOutputProvider` tracks outputs in HashMap
- Outputs can be retrieved by ID for UI rendering
- Code compiles without warnings

