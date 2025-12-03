test instruction

# Test ADDI: 5 + 3 = 8
addi a0, zero, 5
addi a1, a0, 3
ebreak

; check: Registers after execution:
; check: a0 = 5
; check: a1 = 8

# Test ADDI with negative: 10 + (-4) = 6
addi a0, zero, 10
addi a1, a0, -4
ebreak

; check: Registers after execution:
; check: a0 = 10
; check: a1 = 6


