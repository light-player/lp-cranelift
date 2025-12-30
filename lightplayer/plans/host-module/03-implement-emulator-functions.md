# Phase 3: Implement Emulator Functions

## Goal

Add `__host_debug` and `__host_println` implementations in `lp-builtins-app` that use syscalls to send output to the host, similar to the existing `_print` function.

## Tasks

1. Add implementations in `lp-builtins-app/src/main.rs` or new module:
   - `__host_debug`: Check if output should be printed (can be always-on for now, or use a flag)
   - `__host_println`: Always print with newline
   - Both use syscall 2 (write) like existing `_print`

2. Follow existing pattern:
   - Use `BuiltinsWriter` or create similar writer
   - Use `core::fmt::Write` trait
   - Handle `fmt::Arguments` parameter

3. Mark functions with `#[unsafe(no_mangle)]`:
   - Ensure they're exported as `__host_debug` and `__host_println`
   - Linker will resolve these symbols

4. Reference functions to prevent dead code elimination:
   - Add references in `main()` function (like builtins)

## Success Criteria

- Functions compile in `lp-builtins-app`
- Functions are exported with correct names
- Functions can be called from emulator code
- Output appears via syscalls to host

