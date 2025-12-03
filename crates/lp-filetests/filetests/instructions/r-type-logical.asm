test instruction

# Test AND: 0b1010 & 0b1100 = 0b1000 (10 & 12 = 8)
addi a0, zero, 10
addi a1, zero, 12
and a2, a0, a1
ebreak

; check: Registers after execution:
; check: a0 = 10
; check: a1 = 12
; check: a2 = 8

# Test OR: 0b1010 | 0b1100 = 0b1110 (10 | 12 = 14)
addi a0, zero, 10
addi a1, zero, 12
or a2, a0, a1
ebreak

; check: Registers after execution:
; check: a0 = 10
; check: a1 = 12
; check: a2 = 14

# Test XOR: 0b1010 ^ 0b1100 = 0b0110 (10 ^ 12 = 6)
addi a0, zero, 10
addi a1, zero, 12
xor a2, a0, a1
ebreak

; check: Registers after execution:
; check: a0 = 10
; check: a1 = 12
; check: a2 = 6


