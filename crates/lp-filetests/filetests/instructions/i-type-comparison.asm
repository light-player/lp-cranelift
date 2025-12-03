test instruction

# Test SLTI: 5 < 10 = 1 (true)
addi a0, zero, 5
slti a1, a0, 10
ebreak

; check: Registers after execution:
; check: a0 = 5
; check: a1 = 1

# Test SLTI: 10 < 5 = 0 (false)
addi a0, zero, 10
slti a1, a0, 5
ebreak

; check: Registers after execution:
; check: a0 = 10
; check: a1 = 0

# Test SLTIU: 5 < 10 = 1 (unsigned, true)
addi a0, zero, 5
sltiu a1, a0, 10
ebreak

; check: Registers after execution:
; check: a0 = 5
; check: a1 = 1


