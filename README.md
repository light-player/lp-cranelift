# Cranelift for Light Player

This is a fork of [Cranelift](https://cranelift.dev/) (from the [Wasmtime](https://github.com/bytecodealliance/wasmtime) project) adapted for **Light Player** - a system for running GLSL shaders on embedded systems like RISC-V for LED control.

## Purpose

This fork modifies Cranelift's code generation to support compilation of GLSL shader code for resource-constrained embedded targets, enabling real-time visual effects and animations on LED arrays controlled by embedded processors.

## Part of Light Player

Light Player is a larger project focused on bringing GPU-shader-like capabilities to embedded LED control systems. This compiler backend is a key component that enables efficient shader execution on devices with limited memory and processing power.

---

For the original Cranelift documentation, see [cranelift.dev](https://cranelift.dev/).  
For the original Wasmtime project, see [wasmtime.dev](https://wasmtime.dev/).
