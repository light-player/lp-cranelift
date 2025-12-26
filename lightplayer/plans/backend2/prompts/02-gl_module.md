It seems to me there may be a better way.

Have some concept of a ModuleSpec (specifier) enum that encodes what's needed to build a target Module to compile into.
Have a function that takes ModuleSpec and produces the module builder.

struct ModuleSpec would probably be like:
- mod_kind: ModuleKind (Jit | Object)
- arch: Arch

Arch might be a new enum or we can reuse one if we have it, but it would be something like:
- Riscv32(flags)
- Arm64()

and have something like arch

Have GlModule that is similar to ClifModule, but has three things:
- spec (ModuleSpec)
- sourceMap: GlSourceMap
- fns: Map<String, GlFunc>
- module: actual cranelift module

GlFunc would hold information about a specific function:
- sig: GlFnSig
- id: u32


When compiling, the caller of the compiler would pass in the ModuleSpec, either Jit or Object
with the needed info.

Then we would have code for creating a ModuleBuilder from a ModuleSpec.
We would build the GlModule directly, avoiding the need for linking at all.

Then, the fixed32 transform, if required, would use the ModuleSpec to create a new
GlModule that it would copy everything into.

What do you think of the general approach? critique it.