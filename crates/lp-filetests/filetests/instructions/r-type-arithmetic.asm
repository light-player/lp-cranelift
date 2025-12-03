test instruction

# Test ADD: 5 + 3 = 8
addi a0, zero, 5
addi a1, zero, 3
add a2, a0, a1
ebreak

; check: Registers after execution:
; check: a0 = 5
; check: a1 = 3
; check: a2 = 8

# Test SUB: 10 - 4 = 6
addi a0, zero, 10
addi a1, zero, 4
sub a2, a0, a1
ebreak

; check: Registers after execution:
; check: a0 = 10
; check: a1 = 4
; check: a2 = 6

# Test MUL: 7 * 6 = 42
addi a0, zero, 7
addi a1, zero, 6
mul a2, a0, a1
ebreak

; check: Registers after execution:
; check: a0 = 7
; check: a1 = 6
; check: a2 = 42

# Test DIV: 20 / 4 = 5
addi a0, zero, 20
addi a1, zero, 4
div a2, a0, a1
ebreak

; check: Registers after execution:
; check: a0 = 20
; check: a1 = 4
; check: a2 = 5

# Test REM: 17 % 5 = 2
addi a0, zero, 17
addi a1, zero, 5
rem a2, a0, a1
ebreak

; check: Registers after execution:
; check: a0 = 17
; check: a1 = 5
; check: a2 = 2


