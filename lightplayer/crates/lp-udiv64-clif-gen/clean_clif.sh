#!/bin/bash
set -e

# Clean up generated CLIF for use in filetests
# Removes comments, nops, verbose assignments, and fixes target

INPUT="${1:-output/divide64.clif}"
OUTPUT="${2:-output/divide64_clean.clif}"

echo "Cleaning CLIF: $INPUT -> $OUTPUT"

# Remove comments, nops, verbose assignments, and clean up
cat "$INPUT" | \
  # Remove all comment lines
  grep -v "^;" | \
  # Remove nop instructions
  grep -v "nop" | \
  # Keep alias assignments (v3 -> v0) - these are valid CLIF
  # No conversion needed
  # Remove empty lines (we'll add them back selectively)
  sed '/^$/d' | \
  # Fix target from aarch64 to riscv32
  sed 's/target aarch64.*/target riscv32 has_m/' | \
  sed 's/apple_aarch64/riscv32/g' | \
  # Remove or simplify set directives (keep only essential ones)
  grep -v "^set " | \
  # Remove function call declarations (sig0, fn0, etc.) - these are for external calls
  grep -v "^[[:space:]]*sig[0-9]" | \
  grep -v "^[[:space:]]*fn[0-9]" | \
  grep -v "^[[:space:]]*gv[0-9]" | \
  # Remove symbol_value instructions (they reference undefined globals)
  grep -v "symbol_value" | \
  # Keep stack slot declarations (ss0 = explicit_slot) - don't remove these
  # (We're not removing ss lines, they're needed)
  # Remove local variable declarations (the long list) - Cranelift will infer these
  grep -v "^; kind" | \
  grep -v "^; ssa" | \
  grep -v "^; ret" | \
  grep -v "^; arg" | \
  grep -v "^; stack" | \
  grep -v "^; zst" | \
  # Clean up function signature - remove the instance/abi comments
  sed 's/;.*$//' | \
  # Remove trailing whitespace
  sed 's/[[:space:]]*$//' > "$OUTPUT.tmp"

# Convert trap patterns to sentinel returns for debugging
# Pattern: call fnX(vY) followed by trap user1
# Convert to: vY = iconst.i32 0xdeadbeef; return vY; ;    trap user1
# Also handle cases where there's a symbol_value line before the call
# Use perl for better regex support
perl -i.bak2 -pe '
  # Skip symbol_value lines (they will be removed anyway)
  if (/^\s*.*symbol_value/) {
    $_ = "";
    next;
  }
  if (/^\s*call fn\d+\((v\d+)\)/) {
    my $var = $1;
    my $indent = $& =~ /^(\s*)/ ? $1 : "";
    $_ = "${indent}${var} = iconst.i32 0xdeadbeef\n";
    my $next = <>;
    if ($next && $next =~ /^\s*(trap user1)/) {
      my $trap_line = $1;
      my $trap_indent = $next =~ /^(\s*)/ ? $1 : "";
      $_ .= "${indent}return ${var}\n;${trap_indent}trap user1\n";
    } else {
      $_ .= $next if $next;
    }
  }
' "$OUTPUT.tmp"

# Add test directives at the top and fix function signature
{
  echo "test interpret"
  echo "test run"
  echo "target riscv32 has_m"
  echo ""
  # Fix function name to %divide64 and remove target suffix
  sed 's/^function u0:[0-9]*(/function %divide64(/' "$OUTPUT.tmp" | \
    sed 's/ riscv32$/ {/' | \
    sed 's/riscv32 {/{/' | \
    # Remove duplicate target line if present
    grep -v "^target riscv32 has_m$" | \
    # Remove duplicate blank lines
    awk 'NF || prev!="" {print; prev=NF} {prev=NF}' | \
    # Remove duplicate closing braces at the end
    awk '
      {
        if ($0 == "}") {
          brace_count++
          if (brace_count == 1) {
            print
          }
        } else {
          print
        }
      }
      END {
        # Ensure exactly one closing brace
        if (brace_count == 0) {
          print "}"
        }
      }'
} > "$OUTPUT"

rm -f "$OUTPUT.tmp"

echo "Cleaned CLIF written to: $OUTPUT"
echo "Lines: $(wc -l < "$OUTPUT")"
