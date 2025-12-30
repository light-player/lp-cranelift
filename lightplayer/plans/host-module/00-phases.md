# Plan Phases

1. Create Host Module Structure - Add `host` module in `lp-builtins` with function declarations and `HostId` enum
2. Implement Macros - Create `host::debug!` and `host::println!` macros that call underlying functions
3. Implement Emulator Functions - Add `__host_debug` and `__host_println` in `lp-builtins-app` (syscall-based, like existing `_print`)
4. Implement Test Functions - Add test implementations in `lp-builtins` using `std::println!` and `DEBUG=1` check
5. Register Host Functions in JIT - Add host function registration in `GlJitModule` (delegate to `lp-glsl` macros)
6. Register Host Functions in Emulator - Add host function declarations and linking in emulator codegen
7. Integration and Testing - Test all three contexts (emulator, JIT, tests) and verify functionality
8. Cleanup - Remove temporary code, fix warnings, ensure tests pass, format code, remove plan directory

