# Plan Phases

1. Rename main to _init in object loader
2. Update bootstrap code for graceful _init handling
3. Add function address map to GlslEmulatorModule
4. Integrate object file loading into GLSL compilation
5. Run bootstrap init on emulator creation
6. Update call_* methods to support any function
7. Remove main() generation from bootstrap.rs
8. Remove main() test files
9. Update filetest execution to call functions directly
10. Cleanup and testing

