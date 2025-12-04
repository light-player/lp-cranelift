# ESP32-C6 Toy Language JIT Test

This app tests the **toy language JIT compiler running on real RISC-V hardware** (ESP32-C6).

## What It Does

1. Parses a toy language function at runtime
2. Compiles it to RISC-V32 machine code using Cranelift (in no_std!)
3. Executes the JIT-compiled code natively on the ESP32-C6
4. Verifies the result

## Test Code

```toy
fn multiply(a, b) -> (result) {
    result = a * b
}
```

The test compiles this at runtime and executes `multiply(7, 6)` to get `42`.

## Hardware Requirements

- **ESP32-C6** dev board (RISC-V32 with 512KB RAM)
- USB cable for programming/debugging

Note: Despite the directory name `esp32c3-jit-test`, this is configured for **ESP32-C6**. 
The C6 has more RAM and better tooling support.

## Building

```bash
cd apps/esp32c3-jit-test
cargo build --release
```

## Flashing

```bash
cargo espflash flash --release --monitor
```

Or with specific port:

```bash
cargo espflash flash --release --monitor --port /dev/tty.usbserial-*
```

## Expected Output

```
ESP32-C6 Toy Language JIT Test
Testing Cranelift JIT on Real RISC-V Hardware!

Toy Language Source:
fn multiply(a, b) -> (result) {
    result = a * b
}

Step 1: Parsing...
  ✓ Parsing successful
Step 2: Creating RISC-V32 ISA...
  ✓ ISA created
Step 3: Building Cranelift IR and compiling...
  ✓ Compilation successful: 8 bytes
Step 4: Executing JIT-compiled function...
Calling multiply(7, 6)
Result: 42
Expected: 42
======================================
✅ JIT TEST SUCCESS ON REAL HARDWARE!
======================================
```

## Memory Configuration

- **Heap**: 128KB (allocated for Cranelift compilation)
- **Stack**: Default ESP-HAL stack (sufficient for Cranelift)
- **Code**: ~8 bytes for the compiled multiply function

## Technical Details

- **ISA**: RISC-V32IMAC (matches ESP32-C6 core)
- **Compiler**: Cranelift (no_std mode)
- **Optimization**: None (for faster compilation)
- **Target**: `riscv32imac-unknown-none-elf`

This proves that:
- ✅ The toy language works in real no_std environments
- ✅ Cranelift JIT compilation works on actual RISC-V hardware
- ✅ Runtime code generation is practical even on embedded systems
