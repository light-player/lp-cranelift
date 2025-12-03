#!/usr/bin/env python3
"""
Script to adapt RISC-V64 ISLE files to RISC-V32.

This script:
1. Removes RV64-specific instructions (*w variants)
2. Updates type references (I64 -> I32, u64 -> u32)
3. Removes RV64-specific type checks (fits_in_32, ty_int_ref_scalar_64)
"""

import re
import sys
from pathlib import Path

# RV64-specific instruction variants to remove (word-sized operations)
RV64_INSTRUCTIONS = [
    'Addw', 'Subw', 'Sllw', 'Srlw', 'Sraw',  # R-type
    'Mulw', 'Divw', 'Divuw', 'Remw', 'Remuw',  # M extension
    'Addiw', 'Slliw', 'SrliW', 'Sraiw',  # I-type
    'CAddw', 'CSubw', 'CAddiw',  # Compressed
    'AmoaddW', 'AmoswapW', 'AmoxorW', 'AmoandW', 'AmoorW',  # Atomics
    'AmominW', 'AmomaxW', 'AmominuW', 'AmomaxuW',
    'Rolw', 'Rorw', 'Roriw',  # Bit manipulation
    'Packw', 'Cpopw', 'Clzw', 'Ctzw',  # More bit ops
]

def remove_rv64_enum_variants(content):
    """Remove RV64-specific enum variants from type definitions."""
    lines = content.split('\n')
    result = []
    skip_line = False
    
    for line in lines:
        # Check if this line defines an RV64-specific instruction
        if any(f'({inst})' in line for inst in RV64_INSTRUCTIONS):
            # Skip this line
            continue
        result.append(line)
    
    return '\n'.join(result)

def remove_rv64_helper_functions(content):
    """Remove RV64-specific helper function declarations and rules."""
    # Pattern for helper function declarations like (decl rv_addw ...)
    content = re.sub(
        r';;.*rv_\w*w\b.*\n',
        '',
        content
    )
    
    # Remove function declarations for *w instructions
    for inst in RV64_INSTRUCTIONS:
        inst_lower = inst.lower()
        # Remove decl lines
        content = re.sub(
            rf'\(decl rv_{inst_lower} .*?\)\n',
            '',
            content
        )
        # Remove rule lines
        content = re.sub(
            rf'\(rule \(rv_{inst_lower} .*?\n.*?\)\)',
            '',
            content,
            flags=re.DOTALL
        )
    
    return content

def remove_rv64_lowering_rules(content):
    """Remove lowering rules that use RV64-specific instructions."""
    # Remove rules that use rv_*w functions
    for inst in RV64_INSTRUCTIONS:
        inst_lower = inst.lower()
        # Match rules that call rv_*w
        content = re.sub(
            rf'\(rule.*?\(rv_{inst_lower}\b.*?\)\)',
            '',
            content,
            flags=re.DOTALL
        )
    
    return content

def update_type_references(content):
    """Update type references from 64-bit to 32-bit."""
    
    # Update type constants
    replacements = [
        (r'\$I64\b', '$I32'),
        (r'\$F64\b', '$F32'),  # Note: might not want this for all cases
        (r'\bu64_from_imm64\b', 'u32_from_imm32'),
        (r'\bu64_from_ieee64\b', 'u32_from_ieee32'),
        (r'\bi64_sextend_u64\b', 'i32_sextend_u32'),
        (r'\bu64_or\b', 'u32_or'),
        (r'\bu128_low_bits\b', 'u64_low_bits'),  # For I64 on RV32
        (r'\bu128_high_bits\b', 'u64_high_bits'),
    ]
    
    for pattern, replacement in replacements:
        content = re.sub(pattern, replacement, content)
    
    return content

def remove_rv64_type_checks(content):
    """Remove RV64-specific type checking predicates."""
    
    # Remove fits_in_32 checks (not needed on RV32)
    content = re.sub(
        r'\(fits_in_32\s+(\w+)\)',
        r'\1',  # Just use the type directly
        content
    )
    
    # Remove ty_int_ref_scalar_64 (use ty_int_ref_scalar_32 or equivalent)
    content = re.sub(
        r'ty_int_ref_scalar_64',
        'ty_int_ref_scalar_32',
        content
    )
    
    return content

def remove_select_addi_fits_in_32(content):
    """Remove the select_addi rule for fits_in_32 that returns Addiw."""
    content = re.sub(
        r'\(rule \d+ \(select_addi \(fits_in_32 ty\)\) \(AluOPRRI\.Addiw\)\)',
        '',
        content
    )
    return content

def process_isle_file(filepath):
    """Process a single ISLE file."""
    print(f"Processing {filepath}...")
    
    with open(filepath, 'r') as f:
        content = f.read()
    
    original_size = len(content)
    
    # Apply transformations
    content = remove_rv64_enum_variants(content)
    content = remove_rv64_helper_functions(content)
    content = remove_rv64_lowering_rules(content)
    content = remove_select_addi_fits_in_32(content)
    content = update_type_references(content)
    content = remove_rv64_type_checks(content)
    
    # Clean up multiple blank lines
    content = re.sub(r'\n{3,}', '\n\n', content)
    
    new_size = len(content)
    removed = original_size - new_size
    
    print(f"  Removed {removed} characters ({removed // original_size * 100}% reduction)")
    
    # Write back
    with open(filepath, 'w') as f:
        f.write(content)
    
    print(f"  Updated {filepath}")

def main():
    # Get the script directory and find ISLE files
    script_dir = Path(__file__).parent
    riscv32_dir = script_dir.parent / 'cranelift' / 'codegen' / 'src' / 'isa' / 'riscv32'
    
    if not riscv32_dir.exists():
        print(f"Error: {riscv32_dir} not found")
        sys.exit(1)
    
    isle_files = [
        riscv32_dir / 'inst.isle',
        riscv32_dir / 'lower.isle',
        riscv32_dir / 'inst_vector.isle',
    ]
    
    for isle_file in isle_files:
        if isle_file.exists():
            process_isle_file(isle_file)
        else:
            print(f"Warning: {isle_file} not found, skipping")
    
    print("\nDone! ISLE files adapted for RV32.")
    print("Note: Some manual review may be needed for edge cases.")

if __name__ == '__main__':
    main()

