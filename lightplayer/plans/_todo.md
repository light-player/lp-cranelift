

## Issues to investigate

- cranelift only has 64-bit iconst. how are we dealing with this? do we always truncate?

## Optimizations

- don't complie unused functions (esp intrinsics)

## Unfinished work

- `feature/udiv64` more accurate division support in fixed32.