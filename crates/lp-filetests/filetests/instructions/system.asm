test instruction

# Test EBREAK: environment break (halt)
addi a0, zero, 42
ebreak

; check: Registers after execution:
; check: a0 = 42

# Test ECALL: environment call (syscall)
# Note: ECALL behavior depends on syscall handler
# For testing, we just verify ECALL doesn't crash
addi a7, zero, 0
addi a0, zero, 100
ecall
addi a0, zero, 42
ebreak

; check: Registers after execution:
; check: a0 = 42
; check: a7 = 0


