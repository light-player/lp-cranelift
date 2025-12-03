test instruction

# Test LW/SW roundtrip
lui sp, 0x80000
addi sp, sp, 256
addi a0, zero, 42
sw a0, 0(sp)
lw a1, 0(sp)
ebreak

; check: Registers after execution:
; check: a0 = 42
; check: a1 = 42
; check: Memory state (non-zero regions):
; check: 0x80000100: 0x0000002a

# Test LW/SW with offset
lui sp, 0x80000
addi sp, sp, 256
addi a0, zero, 100
sw a0, 4(sp)
lw a1, 4(sp)
ebreak

; check: Registers after execution:
; check: a0 = 100
; check: a1 = 100
; check: Memory state (non-zero regions):
; check: 0x80000104: 0x00000064


