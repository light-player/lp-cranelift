# Cranelift for LightPlayer

This is a fork of [Cranelift](https://cranelift.dev/) (from
the [Wasmtime](https://github.com/bytecodealliance/wasmtime) project) adapted
for [LightPlayer](https://github.com/light-player/lightplayer) -
a project that runs GLSL shaders on microcontrollers to control addressable LED arrays.

## Purpose

This fork adds the following to Cranelift:

- Support `no_std`
- Support RISC-V32 IMAC target
- Reduce memory usage for compatibility with low memory environments

---

For the original Cranelift documentation, see [cranelift.dev](https://cranelift.dev/).  
For the original Wasmtime project, see [wasmtime.dev](https://wasmtime.dev/).
